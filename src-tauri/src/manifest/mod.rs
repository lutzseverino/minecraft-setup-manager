pub mod fetch;
pub mod fingerprint;
pub mod schema;
pub mod validation;

use crate::app_state::InstalledServerSnapshot;
use crate::commands::{InstallPlan, InstallPlanRequest, ServerUpdateStatus};
use crate::manifest::schema::{ManifestResource, ManifestResourceTarget, SetupManifest};
use crate::planner;

pub fn build_install_plan(
    manifest: &SetupManifest,
    request: &InstallPlanRequest,
    update_status: ServerUpdateStatus,
    installed: Option<&InstalledServerSnapshot>,
) -> Result<InstallPlan, String> {
    let profile = manifest
        .profiles
        .iter()
        .find(|profile| profile.id == request.profile)
        .ok_or_else(|| "Choose one of the setup options provided by this server.".to_string())?;
    let server_address = if request.server_address.trim().is_empty() {
        manifest.server.address.to_string()
    } else {
        request.server_address.trim().to_string()
    };
    Ok(InstallPlan {
        server_id: request.server_id.clone(),
        update_status,
        minecraft_version: manifest.minecraft.version.to_string(),
        loader_kind: manifest.minecraft.loader.kind,
        loader_version: manifest.minecraft.loader.version.clone(),
        game_directory_name: manifest.install.game_directory_name.to_string(),
        server_name: manifest.display_name.to_string(),
        server_address,
        launcher: request.launcher,
        profile: request.profile.clone(),
        profile_label: profile.label.clone(),
        recommended_memory_mb: profile.recommended_memory_mb,
        actions: planner::build_action_previews(manifest, request, update_status, installed),
        required_mods: selected_resources(manifest, &request.profile)
            .into_iter()
            .filter(|resource| resource.required)
            .filter(|resource| matches!(resource.target, ManifestResourceTarget::Mods))
            .map(|resource| resource.name.clone())
            .collect(),
        optional_mods: optional_mods_for_profile(manifest, &request.profile),
        warnings: vec![
            "The first setup step creates the separate game folder and setup file.".to_string(),
            format!(
                "{} recommends {} MB memory{}.",
                profile.label,
                profile.recommended_memory_mb,
                if profile.includes_shaders {
                    " and shader support"
                } else {
                    ""
                }
            ),
        ],
    })
}

pub fn selected_resources<'a>(
    manifest: &'a SetupManifest,
    profile: &str,
) -> Vec<&'a ManifestResource> {
    manifest
        .resources
        .iter()
        .filter(|resource| {
            resource.profiles.is_empty()
                || resource
                    .profiles
                    .iter()
                    .any(|profile_id| profile_id == profile)
        })
        .collect()
}

fn optional_mods_for_profile(manifest: &SetupManifest, profile: &str) -> Vec<String> {
    selected_resources(manifest, profile)
        .into_iter()
        .filter(|resource| !resource.required)
        .filter(|resource| matches!(resource.target, ManifestResourceTarget::Mods))
        .map(|resource| resource.name.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::LauncherKind;
    use crate::manifest::schema::{
        ManifestInstall, ManifestLoader, ManifestLoaderKind, ManifestMinecraft,
        ManifestPerformanceProfile, ManifestResourceHashes, ManifestResourceSource,
        ManifestResourceType, ManifestServer,
    };

    #[test]
    fn resource_selection_is_driven_by_manifest_profile_membership() {
        let manifest = manifest_with_resources(vec![
            resource("shared", vec![]),
            resource("light-only", vec!["light"]),
            resource("visual-only", vec!["visual"]),
        ]);

        let selected = selected_resources(&manifest, "light")
            .into_iter()
            .map(|resource| resource.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(selected, vec!["shared", "light-only"]);
    }

    #[test]
    fn install_plan_rejects_a_profile_the_manifest_does_not_define() {
        let manifest = manifest_with_resources(vec![]);
        let request = InstallPlanRequest {
            server_id: "example".to_string(),
            manifest_fingerprint: "sha256:example".to_string(),
            launcher: LauncherKind::Official,
            profile: "missing".to_string(),
            server_address: "play.example.com".to_string(),
        };

        let error = build_install_plan(&manifest, &request, ServerUpdateStatus::NewSetup, None)
            .expect_err("unknown profile must fail");

        assert!(error.contains("Choose one of the setup options"));
    }

    #[test]
    fn install_plan_uses_manifest_profile_details() {
        let manifest = manifest_with_resources(vec![]);
        let request = InstallPlanRequest {
            server_id: "example".to_string(),
            manifest_fingerprint: "sha256:example".to_string(),
            launcher: LauncherKind::Official,
            profile: "light".to_string(),
            server_address: "play.example.com".to_string(),
        };

        let plan = build_install_plan(&manifest, &request, ServerUpdateStatus::NewSetup, None)
            .expect("build plan");

        assert_eq!(plan.profile_label, "Light");
        assert_eq!(plan.recommended_memory_mb, 3072);
    }

    fn manifest_with_resources(resources: Vec<ManifestResource>) -> SetupManifest {
        SetupManifest {
            schema_version: 1,
            manifest_version: "1".to_string(),
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
            profiles: vec![
                ManifestPerformanceProfile {
                    id: "light".to_string(),
                    label: "Light".to_string(),
                    recommended_memory_mb: 3072,
                    includes_shaders: false,
                },
                ManifestPerformanceProfile {
                    id: "visual".to_string(),
                    label: "Visual".to_string(),
                    recommended_memory_mb: 6144,
                    includes_shaders: true,
                },
            ],
            resources,
            server_entry: None,
        }
    }

    fn resource(id: &str, profiles: Vec<&str>) -> ManifestResource {
        ManifestResource {
            id: id.to_string(),
            name: id.to_string(),
            resource_type: ManifestResourceType::Mod,
            target: ManifestResourceTarget::Mods,
            required: true,
            profiles: profiles.into_iter().map(str::to_string).collect(),
            file_name: Some(format!("{id}.jar")),
            source: ManifestResourceSource::Direct {
                url: format!("https://example.com/{id}.jar"),
            },
            hashes: ManifestResourceHashes {
                sha512: None,
                sha256: Some("00".repeat(32)),
            },
        }
    }
}
