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
    let update_status = update_status_for(&server, &manifest, &manifest_fingerprint);

    Ok(ResolvedServerManifest {
        server,
        manifest,
        manifest_fingerprint,
        update_status,
    })
}

#[tauri::command]
pub fn get_install_plan(request: InstallPlanRequest) -> Result<InstallPlan, String> {
    let server = crate::app_state::saved_server_entry(&request.server_id)?;
    let (manifest, manifest_fingerprint) = approved_manifest(&server, &request)?;
    let update_status = update_status_for(&server, &manifest, &manifest_fingerprint);
    let installed = crate::app_state::installed_server_snapshot(&request.server_id)?;

    manifest::build_install_plan(&manifest, &request, update_status, installed.as_ref())
}

#[tauri::command]
pub fn start_install(request: InstallPlanRequest) -> Result<InstallProgress, String> {
    let server = crate::app_state::saved_server_entry(&request.server_id)?;
    let (manifest, manifest_fingerprint) = approved_manifest(&server, &request)?;
    let update_status = update_status_for(&server, &manifest, &manifest_fingerprint);
    let installed = crate::app_state::installed_server_snapshot(&request.server_id)?;
    let plan =
        manifest::build_install_plan(&manifest, &request, update_status, installed.as_ref())?;
    planner::ensure_plan_is_supported(&plan)?;
    let client_setup = setup::prepare_client(&plan, &manifest)?;
    let log = minecraft::install_log(&plan, &client_setup);
    crate::app_state::record_installed_server(
        &request.server_id,
        request.launcher,
        &request.profile,
        client_setup.local_install.game_dir.clone(),
        &manifest,
        &manifest_fingerprint,
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
    let plan = get_install_plan(request)?;
    let validation = setup::validate_client(&plan)?;

    Ok(minecraft::validation::validate_client_setup(
        &plan,
        &validation,
    ))
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

fn update_status_for(
    server: &SavedServerEntry,
    manifest: &manifest::schema::SetupManifest,
    manifest_fingerprint: &str,
) -> ServerUpdateStatus {
    match (
        &server.installed_manifest_version,
        &server.installed_manifest_fingerprint,
    ) {
        (Some(_), Some(fingerprint)) if fingerprint == manifest_fingerprint => {
            ServerUpdateStatus::UpToDate
        }
        (Some(version), None) if version == &manifest.manifest_version => {
            ServerUpdateStatus::UpToDate
        }
        (Some(_), _) => ServerUpdateStatus::UpdateAvailable,
        (None, _) => ServerUpdateStatus::NewSetup,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::schema::{
        ManifestInstall, ManifestLoader, ManifestLoaderKind, ManifestMinecraft, ManifestServer,
        SetupManifest,
    };

    #[test]
    fn update_status_uses_manifest_fingerprint_when_available() {
        let server = saved_server(Some("1"), Some("sha256:old"));
        let manifest = manifest("1");

        assert_eq!(
            ServerUpdateStatus::UpdateAvailable,
            update_status_for(&server, &manifest, "sha256:new")
        );
    }

    #[test]
    fn update_status_keeps_old_version_only_records_compatible() {
        let server = saved_server(Some("1"), None);
        let manifest = manifest("1");

        assert_eq!(
            ServerUpdateStatus::UpToDate,
            update_status_for(&server, &manifest, "sha256:new")
        );
    }

    #[test]
    fn update_status_marks_missing_install_as_new_setup() {
        let server = saved_server(None, None);
        let manifest = manifest("1");

        assert_eq!(
            ServerUpdateStatus::NewSetup,
            update_status_for(&server, &manifest, "sha256:new")
        );
    }

    #[test]
    fn apply_requires_the_exact_reviewed_manifest_fingerprint() {
        assert!(ensure_manifest_was_approved("sha256:same", "sha256:same").is_ok());
        assert!(ensure_manifest_was_approved("sha256:old", "sha256:new").is_err());
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
        }
    }

    fn manifest(version: &str) -> SetupManifest {
        SetupManifest {
            schema_version: 1,
            manifest_version: version.to_string(),
            id: "example".to_string(),
            display_name: "Example".to_string(),
            server: ManifestServer {
                name: "Example".to_string(),
                address: "play.example.com".to_string(),
            },
            minecraft: ManifestMinecraft {
                version: "1.21.6".to_string(),
                loader: ManifestLoader {
                    kind: ManifestLoaderKind::None,
                    version: None,
                },
            },
            install: ManifestInstall {
                game_directory_name: "Example".to_string(),
                launcher_profile_name: "Example".to_string(),
            },
            profiles: vec![],
            resources: vec![],
            server_entry: None,
        }
    }
}
