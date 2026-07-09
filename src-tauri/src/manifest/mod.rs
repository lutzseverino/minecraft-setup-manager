pub mod fetch;
pub mod fingerprint;
pub mod schema;

use crate::commands::{InstallPlan, InstallPlanRequest, PerformanceProfileId};
use crate::performance_profiles::PerformanceProfile;

#[derive(Debug, Clone)]
pub struct ClientManifest {
    pub minecraft_version: &'static str,
    pub fabric_loader_version: &'static str,
    pub game_directory_name: &'static str,
    pub server_name: &'static str,
    pub default_server_address: &'static str,
    pub required_mods: Vec<&'static str>,
    pub balanced_extras: Vec<&'static str>,
    pub shaders_extras: Vec<&'static str>,
}

pub fn load_manifest() -> ClientManifest {
    ClientManifest {
        minecraft_version: "26.1.2",
        fabric_loader_version: "0.19.3",
        game_directory_name: "Maresme MC",
        server_name: "Maresme MC",
        default_server_address: "localhost",
        required_mods: vec![
            "Fabric API",
            "Simple Voice Chat",
            "Sodium",
            "Lithium",
            "ImmediatelyFast",
        ],
        balanced_extras: vec![
            "Sodium Extra",
            "Dynamic FPS",
            "Entity Culling",
            "FerriteCore",
            "Mod Menu",
        ],
        shaders_extras: vec!["Iris", "Reese's Sodium Options"],
    }
}

pub fn build_install_plan(
    manifest: &ClientManifest,
    request: &InstallPlanRequest,
    profile: PerformanceProfile,
) -> InstallPlan {
    let server_address = if request.server_address.trim().is_empty() {
        manifest.default_server_address.to_string()
    } else {
        request.server_address.trim().to_string()
    };

    InstallPlan {
        server_id: request.server_id.clone(),
        minecraft_version: manifest.minecraft_version.to_string(),
        fabric_loader_version: manifest.fabric_loader_version.to_string(),
        game_directory_name: manifest.game_directory_name.to_string(),
        server_name: manifest.server_name.to_string(),
        server_address,
        launcher: request.launcher,
        profile: request.profile,
        steps: vec![
            "fabric_version".to_string(),
            "game_directory".to_string(),
            "launcher_profile".to_string(),
            "mods_directory".to_string(),
            "setup_receipt".to_string(),
            "validation".to_string(),
        ],
        required_mods: manifest
            .required_mods
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        optional_mods: optional_mods_for_profile(manifest, request.profile),
        warnings: vec![
            "The first setup step creates the separate game folder and setup file.".to_string(),
            format!(
                "{:?} / {} profile recommends {} MB memory{}.",
                profile.id,
                profile.label,
                profile.recommended_memory_mb,
                if profile.includes_shaders {
                    " and shader support"
                } else {
                    ""
                }
            ),
        ],
    }
}

fn optional_mods_for_profile(
    manifest: &ClientManifest,
    profile: PerformanceProfileId,
) -> Vec<String> {
    match profile {
        PerformanceProfileId::LowEnd => vec!["Dynamic FPS", "Entity Culling", "FerriteCore"],
        PerformanceProfileId::Balanced => manifest.balanced_extras.clone(),
        PerformanceProfileId::Shaders => manifest
            .balanced_extras
            .iter()
            .chain(manifest.shaders_extras.iter())
            .copied()
            .collect(),
    }
    .into_iter()
    .map(str::to_string)
    .collect()
}
