pub mod fetch;
pub mod fingerprint;
pub mod schema;

use crate::commands::{InstallPlan, InstallPlanRequest, PerformanceProfileId, ServerUpdateStatus};
use crate::manifest::schema::{
    ManifestLoaderKind, ManifestResource, ManifestResourceTarget, SetupManifest,
};
use crate::performance_profiles::PerformanceProfile;
use crate::planner;

pub fn build_install_plan(
    manifest: &SetupManifest,
    request: &InstallPlanRequest,
    profile: PerformanceProfile,
    update_status: ServerUpdateStatus,
) -> InstallPlan {
    let server_address = if request.server_address.trim().is_empty() {
        manifest.server.address.to_string()
    } else {
        request.server_address.trim().to_string()
    };
    let loader_version = match manifest.minecraft.loader.kind {
        ManifestLoaderKind::Fabric => manifest
            .minecraft
            .loader
            .version
            .clone()
            .unwrap_or_default(),
        ManifestLoaderKind::None => String::new(),
    };

    InstallPlan {
        server_id: request.server_id.clone(),
        update_status,
        minecraft_version: manifest.minecraft.version.to_string(),
        fabric_loader_version: loader_version,
        game_directory_name: manifest.install.game_directory_name.to_string(),
        server_name: manifest.display_name.to_string(),
        server_address,
        launcher: request.launcher,
        profile: request.profile,
        actions: planner::build_action_previews(manifest, request, update_status),
        required_mods: manifest
            .resources
            .iter()
            .filter(|resource| resource.required)
            .filter(|resource| matches!(resource.target, ManifestResourceTarget::Mods))
            .map(|resource| resource.name.clone())
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

pub fn selected_resources(
    manifest: &SetupManifest,
    profile: PerformanceProfileId,
) -> Vec<&ManifestResource> {
    manifest
        .resources
        .iter()
        .filter(|resource| {
            resource.required
                || match profile {
                    PerformanceProfileId::LowEnd => false,
                    PerformanceProfileId::Balanced => !resource.id.contains("shader"),
                    PerformanceProfileId::Shaders => true,
                }
        })
        .collect()
}

fn optional_mods_for_profile(
    manifest: &SetupManifest,
    profile: PerformanceProfileId,
) -> Vec<String> {
    let optional_mods = manifest
        .resources
        .iter()
        .filter(|resource| !resource.required)
        .filter(|resource| matches!(resource.target, ManifestResourceTarget::Mods));

    match profile {
        PerformanceProfileId::LowEnd => Vec::new(),
        PerformanceProfileId::Balanced => optional_mods
            .filter(|resource| !resource.id.contains("shader"))
            .map(|resource| resource.name.clone())
            .collect(),
        PerformanceProfileId::Shaders => optional_mods
            .map(|resource| resource.name.clone())
            .collect(),
    }
}
