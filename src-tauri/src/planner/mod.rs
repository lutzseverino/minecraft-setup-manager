use std::collections::HashMap;

use crate::app_state::{InstalledResourceSnapshot, InstalledServerSnapshot};
use crate::commands::{
    InstallPlanRequest, LauncherKind, PerformanceProfileId, ServerUpdateStatus, SetupActionIntent,
    SetupActionKind, SetupActionPreview, SetupActionStatus, SetupActionTarget,
};
use crate::manifest::schema::{
    ManifestLoaderKind, ManifestResource, ManifestResourceTarget, SetupManifest,
};
use crate::manifest::selected_resources;

pub fn build_action_previews(
    manifest: &SetupManifest,
    request: &InstallPlanRequest,
    update_status: ServerUpdateStatus,
    installed: Option<&InstalledServerSnapshot>,
) -> Vec<SetupActionPreview> {
    let mut actions = Vec::new();
    let setup_intent = setup_intent(update_status);

    actions.extend(loader_actions(manifest, setup_intent));
    actions.push(SetupActionPreview {
        id: "game_directory".to_string(),
        kind: SetupActionKind::EnsureGameDirectory,
        intent: setup_intent,
        status: SetupActionStatus::Ready,
        required: true,
        resource_id: None,
        subject: Some(manifest.install.game_directory_name.clone()),
        target: None,
    });
    actions.push(SetupActionPreview {
        id: "launcher_profile".to_string(),
        kind: SetupActionKind::EnsureLauncherProfile,
        intent: setup_intent,
        status: launcher_action_status(request.launcher),
        required: matches!(request.launcher, LauncherKind::Official),
        resource_id: None,
        subject: Some(manifest.install.launcher_profile_name.clone()),
        target: None,
    });

    actions.extend(resource_actions(manifest, request.profile, installed));

    if let Some(server_entry) = &manifest.server_entry {
        actions.push(SetupActionPreview {
            id: "server_entry".to_string(),
            kind: SetupActionKind::WriteServerEntry,
            intent: setup_intent,
            status: SetupActionStatus::NotImplemented,
            required: false,
            resource_id: None,
            subject: Some(server_entry.name.clone()),
            target: None,
        });
    }

    actions.push(SetupActionPreview {
        id: "setup_receipt".to_string(),
        kind: SetupActionKind::WriteSetupReceipt,
        intent: setup_intent,
        status: SetupActionStatus::Ready,
        required: true,
        resource_id: None,
        subject: None,
        target: None,
    });
    actions.push(SetupActionPreview {
        id: "validation".to_string(),
        kind: SetupActionKind::ValidateSetup,
        intent: SetupActionIntent::Verify,
        status: SetupActionStatus::Ready,
        required: true,
        resource_id: None,
        subject: None,
        target: None,
    });

    actions
}

fn loader_actions(
    manifest: &SetupManifest,
    setup_intent: SetupActionIntent,
) -> Vec<SetupActionPreview> {
    match manifest.minecraft.loader.kind {
        ManifestLoaderKind::None => vec![SetupActionPreview {
            id: "loader_none".to_string(),
            kind: SetupActionKind::VerifyLoader,
            intent: SetupActionIntent::Verify,
            status: SetupActionStatus::Ready,
            required: true,
            resource_id: None,
            subject: Some(manifest.minecraft.version.clone()),
            target: None,
        }],
        ManifestLoaderKind::Fabric => {
            let version = manifest
                .minecraft
                .loader
                .version
                .clone()
                .unwrap_or_else(|| "unknown".to_string());

            vec![
                SetupActionPreview {
                    id: "fabric_version".to_string(),
                    kind: SetupActionKind::VerifyLoader,
                    intent: SetupActionIntent::Verify,
                    status: SetupActionStatus::Ready,
                    required: true,
                    resource_id: None,
                    subject: Some(version),
                    target: None,
                },
                SetupActionPreview {
                    id: "fabric_install".to_string(),
                    kind: SetupActionKind::InstallLoader,
                    intent: setup_intent,
                    status: SetupActionStatus::NotImplemented,
                    required: true,
                    resource_id: None,
                    subject: Some(manifest.minecraft.version.clone()),
                    target: None,
                },
            ]
        }
    }
}

fn resource_actions(
    manifest: &SetupManifest,
    profile: PerformanceProfileId,
    installed: Option<&InstalledServerSnapshot>,
) -> Vec<SetupActionPreview> {
    let installed_resources = installed_resource_map(installed);

    selected_resources(manifest, profile)
        .into_iter()
        .map(|resource| {
            let installed_resource = installed_resources.get(resource.id.as_str());

            SetupActionPreview {
                id: format!("resource_{}", resource.id),
                kind: SetupActionKind::SyncResource,
                intent: resource_intent(resource, installed_resource),
                status: SetupActionStatus::NotImplemented,
                required: resource.required,
                resource_id: Some(resource.id.clone()),
                subject: Some(resource.name.clone()),
                target: Some(setup_action_target(resource.target.clone())),
            }
        })
        .collect()
}

fn installed_resource_map(
    installed: Option<&InstalledServerSnapshot>,
) -> HashMap<&str, &InstalledResourceSnapshot> {
    installed
        .map(|snapshot| {
            snapshot
                .resources
                .iter()
                .map(|resource| (resource.id.as_str(), resource))
                .collect()
        })
        .unwrap_or_default()
}

fn resource_intent(
    resource: &ManifestResource,
    installed: Option<&&InstalledResourceSnapshot>,
) -> SetupActionIntent {
    match installed {
        None => SetupActionIntent::Add,
        Some(installed)
            if installed.target == resource.target
                && installed.source == resource.source
                && installed.hashes == resource.hashes =>
        {
            SetupActionIntent::Verify
        }
        Some(_) => SetupActionIntent::Update,
    }
}

fn setup_action_target(target: ManifestResourceTarget) -> SetupActionTarget {
    match target {
        ManifestResourceTarget::Mods => SetupActionTarget::Mods,
        ManifestResourceTarget::Resourcepacks => SetupActionTarget::Resourcepacks,
        ManifestResourceTarget::Shaderpacks => SetupActionTarget::Shaderpacks,
        ManifestResourceTarget::Config => SetupActionTarget::Config,
    }
}

fn launcher_action_status(launcher: LauncherKind) -> SetupActionStatus {
    match launcher {
        LauncherKind::Official => SetupActionStatus::Ready,
        LauncherKind::Sklauncher | LauncherKind::Manual => SetupActionStatus::NotImplemented,
    }
}

fn setup_intent(update_status: ServerUpdateStatus) -> SetupActionIntent {
    match update_status {
        ServerUpdateStatus::NewSetup => SetupActionIntent::Add,
        ServerUpdateStatus::UpdateAvailable => SetupActionIntent::Update,
        ServerUpdateStatus::UpToDate => SetupActionIntent::Verify,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::{InstalledResourceSnapshot, InstalledServerSnapshot};
    use crate::manifest::schema::{
        ManifestInstall, ManifestLoader, ManifestLoaderKind, ManifestMinecraft, ManifestResource,
        ManifestResourceHashes, ManifestResourceSource, ManifestResourceTarget,
        ManifestResourceType, ManifestServer,
    };

    #[test]
    fn resource_actions_mark_matching_installed_resources_as_verify() {
        let manifest = manifest_with_resource(resource(
            "fabric-api",
            ManifestResourceSource::Modrinth {
                project: "fabric-api".to_string(),
                version: "1.0.0".to_string(),
            },
            hashes("abc"),
        ));
        let installed = installed_snapshot(vec![installed_resource(
            "fabric-api",
            ManifestResourceSource::Modrinth {
                project: "fabric-api".to_string(),
                version: "1.0.0".to_string(),
            },
            hashes("abc"),
        )]);

        let actions = build_action_previews(
            &manifest,
            &request(),
            ServerUpdateStatus::UpToDate,
            Some(&installed),
        );

        assert_eq!(
            Some(SetupActionIntent::Verify),
            resource_action(&actions, "fabric-api").map(|action| action.intent)
        );
    }

    #[test]
    fn resource_actions_mark_changed_resources_as_update() {
        let manifest = manifest_with_resource(resource(
            "fabric-api",
            ManifestResourceSource::Modrinth {
                project: "fabric-api".to_string(),
                version: "2.0.0".to_string(),
            },
            hashes("new"),
        ));
        let installed = installed_snapshot(vec![installed_resource(
            "fabric-api",
            ManifestResourceSource::Modrinth {
                project: "fabric-api".to_string(),
                version: "1.0.0".to_string(),
            },
            hashes("old"),
        )]);

        let actions = build_action_previews(
            &manifest,
            &request(),
            ServerUpdateStatus::UpdateAvailable,
            Some(&installed),
        );

        assert_eq!(
            Some(SetupActionIntent::Update),
            resource_action(&actions, "fabric-api").map(|action| action.intent)
        );
    }

    #[test]
    fn resource_actions_mark_new_resources_as_add() {
        let manifest = manifest_with_resource(resource(
            "fabric-api",
            ManifestResourceSource::Modrinth {
                project: "fabric-api".to_string(),
                version: "1.0.0".to_string(),
            },
            hashes("abc"),
        ));
        let installed = installed_snapshot(vec![]);

        let actions = build_action_previews(
            &manifest,
            &request(),
            ServerUpdateStatus::UpdateAvailable,
            Some(&installed),
        );

        assert_eq!(
            Some(SetupActionIntent::Add),
            resource_action(&actions, "fabric-api").map(|action| action.intent)
        );
    }

    fn resource_action<'a>(
        actions: &'a [SetupActionPreview],
        resource_id: &str,
    ) -> Option<&'a SetupActionPreview> {
        actions
            .iter()
            .find(|action| action.resource_id.as_deref() == Some(resource_id))
    }

    fn request() -> InstallPlanRequest {
        InstallPlanRequest {
            server_id: "example".to_string(),
            launcher: LauncherKind::Official,
            profile: PerformanceProfileId::Balanced,
            server_address: "play.example.com".to_string(),
        }
    }

    fn manifest_with_resource(resource: ManifestResource) -> SetupManifest {
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
            profiles: vec![],
            resources: vec![resource],
            server_entry: None,
        }
    }

    fn resource(
        id: &str,
        source: ManifestResourceSource,
        hashes: ManifestResourceHashes,
    ) -> ManifestResource {
        ManifestResource {
            id: id.to_string(),
            name: "Fabric API".to_string(),
            resource_type: ManifestResourceType::Mod,
            target: ManifestResourceTarget::Mods,
            required: true,
            source,
            hashes,
        }
    }

    fn installed_snapshot(resources: Vec<InstalledResourceSnapshot>) -> InstalledServerSnapshot {
        InstalledServerSnapshot { resources }
    }

    fn installed_resource(
        id: &str,
        source: ManifestResourceSource,
        hashes: ManifestResourceHashes,
    ) -> InstalledResourceSnapshot {
        InstalledResourceSnapshot {
            id: id.to_string(),
            target: ManifestResourceTarget::Mods,
            source,
            hashes,
        }
    }

    fn hashes(sha256: &str) -> ManifestResourceHashes {
        ManifestResourceHashes {
            sha512: None,
            sha256: Some(sha256.to_string()),
        }
    }
}
