mod types;

pub use types::*;

use crate::launcher;
use crate::manifest;
use crate::minecraft;
use crate::performance_profiles;
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
    let update_status = match &server.installed_manifest_version {
        Some(version) if version == &manifest.manifest_version => ServerUpdateStatus::UpToDate,
        Some(_) => ServerUpdateStatus::UpdateAvailable,
        None => ServerUpdateStatus::NewSetup,
    };

    Ok(ResolvedServerManifest {
        server,
        manifest,
        manifest_fingerprint,
        update_status,
    })
}

#[tauri::command]
pub fn get_install_plan(request: InstallPlanRequest) -> Result<InstallPlan, String> {
    let (manifest, _) = saved_manifest(&request.server_id)?;
    let profile = performance_profiles::resolve_profile(&request.profile);

    Ok(manifest::build_install_plan(&manifest, &request, profile))
}

#[tauri::command]
pub fn start_install(request: InstallPlanRequest) -> Result<InstallProgress, String> {
    let (manifest, manifest_fingerprint) = saved_manifest(&request.server_id)?;
    let profile = performance_profiles::resolve_profile(&request.profile);
    let plan = manifest::build_install_plan(&manifest, &request, profile);
    let client_setup = setup::prepare_client(&plan)?;
    let log = minecraft::install_log(&plan, &client_setup);
    crate::app_state::record_installed_server(
        &request.server_id,
        request.launcher,
        request.profile,
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

fn saved_manifest(server_id: &str) -> Result<(manifest::schema::SetupManifest, String), String> {
    let manifest_url = crate::app_state::saved_server_manifest_url(server_id)?;
    let manifest = manifest::fetch::fetch_manifest(&manifest_url)?;
    let manifest_fingerprint = manifest::fingerprint::manifest_fingerprint(&manifest)?;

    Ok((manifest, manifest_fingerprint))
}
