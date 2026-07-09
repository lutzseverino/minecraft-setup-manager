use crate::commands::InstallPlan;
use crate::launcher::{self, LauncherProfileResult, LauncherProfileValidation};
use crate::manifest::schema::SetupManifest;
use crate::minecraft::fabric_installer::{self, LoaderInstallResult};
use crate::minecraft::local_install::{self, LocalInstallResult, LocalValidationResult};

#[derive(Debug, Clone)]
pub struct ClientSetupResult {
    pub loader_install: LoaderInstallResult,
    pub local_install: LocalInstallResult,
    pub launcher_profile: LauncherProfileResult,
}

#[derive(Debug, Clone)]
pub struct ClientSetupValidation {
    pub local_install: LocalValidationResult,
    pub launcher_profile: LauncherProfileValidation,
}

pub fn prepare_client(
    plan: &InstallPlan,
    manifest: &SetupManifest,
) -> Result<ClientSetupResult, String> {
    launcher::validate_base_prerequisites(plan)?;
    let loader_install = fabric_installer::ensure_loader(plan)?;
    launcher::validate_profile_prerequisites(plan)?;

    let local_install = local_install::prepare_local_install(plan, manifest)?;
    let launcher_profile = launcher::ensure_profile(plan, &local_install.game_dir)?;

    Ok(ClientSetupResult {
        loader_install,
        local_install,
        launcher_profile,
    })
}

pub fn validate_client(plan: &InstallPlan) -> Result<ClientSetupValidation, String> {
    let local_install = local_install::validate_local_install(plan)?;
    let launcher_profile = launcher::validate_profile(plan, &local_install.game_dir)?;

    Ok(ClientSetupValidation {
        local_install,
        launcher_profile,
    })
}
