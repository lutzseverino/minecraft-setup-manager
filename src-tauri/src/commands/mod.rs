mod types;

pub use types::*;

use crate::launcher;
use crate::manifest;
use crate::minecraft;
use crate::planner;
use crate::server;
use crate::setup;

#[tauri::command]
pub fn detect_launchers() -> Vec<LauncherDetection> {
    launcher::detect_launchers()
}

#[tauri::command]
pub fn list_saved_servers() -> Result<Vec<SavedServerEntry>, String> {
    crate::app_state::list_saved_servers()
}

#[tauri::command]
pub fn resolve_server_manifest(
    request: ResolveServerManifestRequest,
) -> Result<ResolvedServerManifest, String> {
    let (address, manifest_url) = server::discovery::manifest_url_for_address(&request.address)?;
    let manifest = manifest::fetch::fetch_manifest(&manifest_url)?;
    let manifest_fingerprint = manifest::fingerprint::manifest_fingerprint(&manifest)?;
    let server = crate::app_state::upsert_checked_server(
        &address,
        &manifest_url,
        &manifest,
        &manifest_fingerprint,
    )?;
    let update_status = update_status_for(&server, &manifest_fingerprint);

    Ok(ResolvedServerManifest {
        server,
        manifest,
        manifest_fingerprint,
        update_status,
    })
}

#[tauri::command]
pub fn get_install_plan(request: InstallPlanRequest) -> Result<InstallPlan, String> {
    let (plan, _, _) = build_plan_context(&request)?;
    Ok(plan)
}

#[tauri::command]
pub fn start_install(request: InstallPlanRequest) -> Result<InstallProgress, String> {
    let (plan, manifest, manifest_fingerprint) = build_plan_context(&request)?;
    planner::ensure_plan_is_supported(&plan)?;
    let client_setup = setup::prepare_client(&plan, &manifest)?;
    let validation = setup::validate_client(&plan, &manifest)?;
    let validation_result = minecraft::validation::validate_client_setup(&plan, &validation);
    ensure_validation_passed(&validation_result)?;
    let installed_resources =
        setup::installed_resources(&manifest, &request.profile, &client_setup.local_install)?;
    let log = minecraft::install_log(&plan, &client_setup);
    crate::app_state::record_installed_server(
        &request.server_id,
        request.launcher,
        &request.profile,
        client_setup.local_install.game_dir.clone(),
        &manifest,
        &manifest_fingerprint,
        installed_resources,
    )?;

    Ok(InstallProgress {
        phase: InstallPhase::Complete,
        percent: 100,
        log,
        plan,
    })
}

#[tauri::command]
pub fn validate_installation(request: InstallPlanRequest) -> Result<ValidationResult, String> {
    let (plan, manifest, _) = build_plan_context(&request)?;
    let validation = setup::validate_client(&plan, &manifest)?;

    Ok(minecraft::validation::validate_client_setup(
        &plan,
        &validation,
    ))
}

#[tauri::command]
pub fn redeem_setup_attestation(
    request: RedeemSetupAttestationRequest,
) -> Result<SetupAttestationReceipt, String> {
    let install_request = InstallPlanRequest {
        server_id: request.server_id,
        manifest_fingerprint: request.manifest_fingerprint,
        launcher: request.launcher,
        profile: request.profile,
    };
    let (plan, manifest, fingerprint) = build_plan_context(&install_request)?;
    planner::ensure_plan_is_supported(&plan)?;
    let validation = setup::validate_client(&plan, &manifest)?;
    let result = minecraft::validation::validate_client_setup(&plan, &validation);
    ensure_validation_passed(&result)?;
    if result.overall != ValidationStatus::Pass {
        return Err(
            "The local setup has warnings, so it cannot be confirmed to the server yet."
                .to_string(),
        );
    }
    let saved_server = crate::app_state::saved_server_entry(&install_request.server_id)?;
    server::attestation::redeem(
        &saved_server.manifest_url,
        &request.challenge,
        &fingerprint,
        &install_request.profile,
    )?;

    Ok(SetupAttestationReceipt {
        manifest_fingerprint: fingerprint,
    })
}

fn build_plan_context(
    request: &InstallPlanRequest,
) -> Result<(InstallPlan, manifest::schema::SetupManifest, String), String> {
    let server = crate::app_state::saved_server_entry(&request.server_id)?;
    let (manifest, manifest_fingerprint) = approved_manifest(&server, request)?;
    let update_status = update_status_for(&server, &manifest_fingerprint);
    let installed = crate::app_state::installed_server_snapshot(&request.server_id)?;
    let plan = manifest::build_install_plan(&manifest, request, update_status, installed.as_ref())?;
    Ok((plan, manifest, manifest_fingerprint))
}

#[tauri::command]
pub fn export_diagnostics() -> Result<DiagnosticBundle, String> {
    minecraft::local_install::export_install_report()
}

fn approved_manifest(
    server: &SavedServerEntry,
    request: &InstallPlanRequest,
) -> Result<(manifest::schema::SetupManifest, String), String> {
    let manifest = crate::app_state::saved_manifest_snapshot(&server.id)?;
    let manifest_fingerprint = manifest::fingerprint::manifest_fingerprint(&manifest)?;

    ensure_manifest_was_approved(&request.manifest_fingerprint, &manifest_fingerprint)?;

    Ok((manifest, manifest_fingerprint))
}

fn ensure_manifest_was_approved(expected: &str, actual: &str) -> Result<(), String> {
    if expected == actual {
        Ok(())
    } else {
        Err(
            "The server setup changed after you reviewed it. Check the server and review the steps again."
                .to_string(),
        )
    }
}

fn ensure_validation_passed(result: &ValidationResult) -> Result<(), String> {
    if result.overall != ValidationStatus::Fail {
        return Ok(());
    }

    let failed = result
        .checks
        .iter()
        .filter(|check| check.status == ValidationStatus::Fail)
        .map(|check| check.label.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!(
        "Setup changed the local files, but these checks still failed: {failed}. Run setup again to repair them."
    ))
}

fn update_status_for(server: &SavedServerEntry, manifest_fingerprint: &str) -> ServerUpdateStatus {
    if server.needs_repair {
        return ServerUpdateStatus::UpdateAvailable;
    }

    match (
        &server.installed_manifest_version,
        &server.installed_manifest_fingerprint,
    ) {
        (Some(_), Some(fingerprint)) if fingerprint == manifest_fingerprint => {
            ServerUpdateStatus::UpToDate
        }
        (Some(_), _) => ServerUpdateStatus::UpdateAvailable,
        (None, _) => ServerUpdateStatus::NewSetup,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_status_uses_manifest_fingerprint_when_available() {
        let server = saved_server(Some("1"), Some("sha256:old"));
        assert_eq!(
            ServerUpdateStatus::UpdateAvailable,
            update_status_for(&server, "sha256:new")
        );
    }

    #[test]
    fn update_status_requires_legacy_records_to_be_verified_once() {
        let server = saved_server(Some("1"), None);

        assert_eq!(
            ServerUpdateStatus::UpdateAvailable,
            update_status_for(&server, "sha256:new")
        );
    }

    #[test]
    fn update_status_marks_missing_install_as_new_setup() {
        let server = saved_server(None, None);
        assert_eq!(
            ServerUpdateStatus::NewSetup,
            update_status_for(&server, "sha256:new")
        );
    }

    #[test]
    fn update_status_marks_old_install_layouts_for_repair() {
        let mut server = saved_server(Some("1"), Some("sha256:same"));
        server.needs_repair = true;

        assert_eq!(
            ServerUpdateStatus::UpdateAvailable,
            update_status_for(&server, "sha256:same")
        );
    }

    #[test]
    fn apply_requires_the_exact_reviewed_manifest_fingerprint() {
        assert!(ensure_manifest_was_approved("sha256:same", "sha256:same").is_ok());
        assert!(ensure_manifest_was_approved("sha256:old", "sha256:new").is_err());
    }

    #[test]
    fn failed_validation_cannot_be_recorded_as_complete() {
        let result = ValidationResult {
            overall: ValidationStatus::Fail,
            checks: vec![ValidationCheck {
                id: "files".to_string(),
                label: "Setup files".to_string(),
                detail: "Missing".to_string(),
                status: ValidationStatus::Fail,
            }],
        };

        assert!(ensure_validation_passed(&result).is_err());
    }

    fn saved_server(
        installed_manifest_version: Option<&str>,
        installed_manifest_fingerprint: Option<&str>,
    ) -> SavedServerEntry {
        SavedServerEntry {
            id: "example".to_string(),
            address: "play.example.com".to_string(),
            manifest_url:
                "https://play.example.com/.well-known/minecraft-setup-manager/manifest.json"
                    .to_string(),
            display_name: "Example".to_string(),
            last_checked_at: "2026-07-09T00:00:00Z".to_string(),
            last_installed_at: installed_manifest_version
                .map(|_| "2026-07-09T00:00:00Z".to_string()),
            selected_launcher: LauncherKind::Official,
            selected_profile: "balanced".to_string(),
            installed_manifest_version: installed_manifest_version.map(str::to_string),
            installed_manifest_fingerprint: installed_manifest_fingerprint.map(str::to_string),
            needs_repair: false,
        }
    }
}
