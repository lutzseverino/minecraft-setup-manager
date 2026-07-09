mod types;

pub use types::*;

use crate::launcher;
use crate::manifest;
use crate::minecraft;
use crate::performance_profiles;
use crate::setup;

#[tauri::command]
pub fn detect_launchers() -> Vec<LauncherDetection> {
    launcher::detect_launchers()
}

#[tauri::command]
pub fn get_install_plan(request: InstallPlanRequest) -> InstallPlan {
    let manifest = manifest::load_manifest();
    let profile = performance_profiles::resolve_profile(&request.profile);

    manifest::build_install_plan(&manifest, &request, profile)
}

#[tauri::command]
pub fn start_install(request: InstallPlanRequest) -> Result<InstallProgress, String> {
    let plan = get_install_plan(request);
    let client_setup = setup::prepare_client(&plan)?;
    let log = minecraft::install_log(&plan, &client_setup);

    Ok(InstallProgress {
        phase: InstallPhase::Complete,
        percent: 100,
        log,
        plan,
    })
}

#[tauri::command]
pub fn validate_installation(request: InstallPlanRequest) -> Result<ValidationResult, String> {
    let plan = get_install_plan(request);
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
