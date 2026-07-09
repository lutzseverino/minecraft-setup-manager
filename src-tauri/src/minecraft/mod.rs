pub mod fabric_installer;
pub mod local_install;
pub mod managed_resources;
pub mod modrinth;
pub mod servers_dat;
pub mod validation;
pub mod version;

use crate::commands::InstallPlan;
use crate::launcher::LauncherProfileAction;
use crate::manifest::schema::ManifestLoaderKind;
use crate::minecraft::fabric_installer::LoaderInstallAction;
use crate::setup::ClientSetupResult;

pub fn install_log(plan: &InstallPlan, client_setup: &ClientSetupResult) -> Vec<String> {
    let mut log = vec![
        loader_log_line(plan),
        format!("[launcher] Selected adapter: {:?}", plan.launcher),
        format!(
            "[fabric] Verified launcher version: {}",
            client_setup.launcher_profile.version_id
        ),
        format!(
            "[folder] Separate game folder: {}",
            client_setup.local_install.game_dir.display()
        ),
        format!(
            "[receipt] Setup file: {}",
            client_setup.local_install.receipt_path.display()
        ),
        format!(
            "[profile] {} ({})",
            profile_action_label(&client_setup.launcher_profile.action),
            client_setup.launcher_profile.profile_id
        ),
        format!(
            "[profile] Game folder: {}",
            client_setup.launcher_profile.game_dir.display()
        ),
        format!(
            "[mods] Required mods on the list: {}",
            plan.required_mods.len()
        ),
        format!("[server] Saved server address: {}", plan.server_address),
    ];

    log.push(format!(
        "[loader] {} {} at {}",
        loader_action_label(&client_setup.loader_install.action),
        client_setup.loader_install.version_id,
        client_setup.loader_install.profile_path.display()
    ));
    log.extend(client_setup.local_install.log.clone());
    log.extend(client_setup.launcher_profile.log.clone());
    if let Some(path) = &client_setup.launcher_profile.launcher_profiles_path {
        log.push(format!(
            "[profile] Launcher profiles file: {}",
            path.display()
        ));
    }
    if let Some(path) = &client_setup.launcher_profile.backup_path {
        log.push(format!(
            "[profile] Launcher profile backup: {}",
            path.display()
        ));
    }
    log
}

fn loader_action_label(action: &LoaderInstallAction) -> &'static str {
    match action {
        LoaderInstallAction::NotNeeded => "Using regular Minecraft version",
        LoaderInstallAction::Installed => "Installed Fabric version",
        LoaderInstallAction::Unchanged => "Fabric version already ready",
    }
}

fn loader_log_line(plan: &InstallPlan) -> String {
    match plan.loader_kind {
        ManifestLoaderKind::None => {
            format!("[manifest] Vanilla Minecraft {}", plan.minecraft_version)
        }
        ManifestLoaderKind::Fabric => format!(
            "[manifest] Minecraft {} with Fabric {}",
            plan.minecraft_version,
            plan.loader_version.as_deref().unwrap_or("unknown")
        ),
    }
}

fn profile_action_label(action: &LauncherProfileAction) -> &'static str {
    match action {
        LauncherProfileAction::Created => "Created launcher profile",
        LauncherProfileAction::Updated => "Updated launcher profile",
        LauncherProfileAction::Unchanged => "Launcher profile already ready",
        LauncherProfileAction::Skipped => "Skipped launcher profile",
    }
}
