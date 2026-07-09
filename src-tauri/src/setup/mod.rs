use crate::app_state::InstalledResourceSnapshot;
use crate::commands::InstallPlan;
use crate::launcher::{self, LauncherProfileResult, LauncherProfileValidation};
use crate::manifest::schema::SetupManifest;
use crate::minecraft::fabric_installer::{self, LoaderInstallResult};
use crate::minecraft::local_install::{self, LocalInstallResult, LocalValidationResult};
use crate::minecraft::managed_resources::ManagedResourceAction;
use crate::minecraft::servers_dat::{self, ServerEntryResult, ServerEntryValidation};

#[derive(Debug, Clone)]
pub struct ClientSetupResult {
    pub loader_install: LoaderInstallResult,
    pub local_install: LocalInstallResult,
    pub server_entry: ServerEntryResult,
    pub launcher_profile: LauncherProfileResult,
}

#[derive(Debug, Clone)]
pub struct ClientSetupValidation {
    pub local_install: LocalValidationResult,
    pub server_entry: ServerEntryValidation,
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
    let server_entry =
        servers_dat::ensure_server_entry(&local_install.game_dir, manifest.server_entry.as_ref())?;
    let launcher_profile = launcher::ensure_profile(plan, &local_install.game_dir)?;

    Ok(ClientSetupResult {
        loader_install,
        local_install,
        server_entry,
        launcher_profile,
    })
}

pub fn validate_client(
    plan: &InstallPlan,
    manifest: &SetupManifest,
) -> Result<ClientSetupValidation, String> {
    let local_install = local_install::validate_local_install(plan)?;
    let server_entry = servers_dat::validate_server_entry(
        &local_install.game_dir,
        manifest.server_entry.as_ref(),
    )?;
    let launcher_profile = launcher::validate_profile(plan, &local_install.game_dir)?;

    Ok(ClientSetupValidation {
        local_install,
        server_entry,
        launcher_profile,
    })
}

pub fn installed_resources(
    manifest: &SetupManifest,
    profile: &str,
    local_install: &LocalInstallResult,
) -> Result<Vec<InstalledResourceSnapshot>, String> {
    crate::manifest::selected_resources(manifest, profile)
        .into_iter()
        .map(|resource| {
            let result = local_install
                .resource_results
                .iter()
                .find(|result| {
                    result.resource_id == resource.id
                        && matches!(
                            result.action,
                            ManagedResourceAction::Downloaded | ManagedResourceAction::Verified
                        )
                })
                .ok_or_else(|| {
                    format!(
                        "Managed resource {} was not verified after setup.",
                        resource.id
                    )
                })?;
            let file_name = result.file_name.clone().ok_or_else(|| {
                format!(
                    "Managed resource {} has no installed file name.",
                    resource.id
                )
            })?;
            let hashes = result.hashes.clone().ok_or_else(|| {
                format!(
                    "Managed resource {} has no verified file hash.",
                    resource.id
                )
            })?;

            Ok(InstalledResourceSnapshot {
                id: resource.id.clone(),
                name: resource.name.clone(),
                target: resource.target.clone(),
                file_name: Some(file_name),
                source: resource.source.clone(),
                hashes,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::manifest::schema::{
        ManifestInstall, ManifestLoader, ManifestLoaderKind, ManifestMinecraft,
        ManifestPerformanceProfile, ManifestResource, ManifestResourceHashes,
        ManifestResourceSource, ManifestResourceTarget, ManifestResourceType, ManifestServer,
    };
    use crate::minecraft::managed_resources::ManagedResourceResult;

    #[test]
    fn installed_state_uses_resolved_file_metadata() {
        let hashes = ManifestResourceHashes {
            sha512: Some("a".repeat(128)),
            sha256: None,
        };
        let local_install = LocalInstallResult {
            game_dir: PathBuf::from("game"),
            receipt_path: PathBuf::from("receipt"),
            resource_results: vec![ManagedResourceResult {
                resource_id: "fabric-api".to_string(),
                action: ManagedResourceAction::Downloaded,
                path: Some(PathBuf::from("game/mods/resolved.jar")),
                file_name: Some("resolved.jar".to_string()),
                hashes: Some(hashes.clone()),
            }],
            log: vec![],
        };

        let resources = installed_resources(&manifest(), "balanced", &local_install)
            .expect("build installed state");

        assert_eq!(resources[0].file_name.as_deref(), Some("resolved.jar"));
        assert_eq!(resources[0].hashes, hashes);
    }

    fn manifest() -> SetupManifest {
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
                    kind: ManifestLoaderKind::Fabric,
                    version: Some("0.16.14".to_string()),
                },
            },
            install: ManifestInstall {
                game_directory_name: "Example".to_string(),
                launcher_profile_name: "Example".to_string(),
            },
            profiles: vec![ManifestPerformanceProfile {
                id: "balanced".to_string(),
                label: "Balanced".to_string(),
                recommended_memory_mb: 4096,
                includes_shaders: false,
            }],
            resources: vec![ManifestResource {
                id: "fabric-api".to_string(),
                name: "Fabric API".to_string(),
                resource_type: ManifestResourceType::Mod,
                target: ManifestResourceTarget::Mods,
                required: true,
                profiles: vec![],
                file_name: None,
                source: ManifestResourceSource::Modrinth {
                    project: "P7dR8mSH".to_string(),
                    version: "F5TVHWcE".to_string(),
                },
                hashes: ManifestResourceHashes::default(),
            }],
            server_entry: None,
        }
    }
}
