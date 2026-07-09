use std::collections::{HashMap, HashSet};

use crate::app_state::{InstalledResourceSnapshot, InstalledServerSnapshot};
use crate::commands::{
    InstallPlanRequest, LauncherKind, ServerUpdateStatus, SetupActionIntent, SetupActionKind,
    SetupActionPreview, SetupActionStatus,
};
use crate::manifest::schema::{ManifestLoaderKind, ManifestResource, SetupManifest};
use crate::manifest::selected_resources;
use crate::minecraft::managed_resources::{
    can_sync_resource, managed_file_name, setup_action_target,
};

pub fn ensure_plan_is_supported(plan: &crate::commands::InstallPlan) -> Result<(), String> {
    let unsupported = plan
        .actions
        .iter()
        .filter(|action| matches!(action.status, SetupActionStatus::NotImplemented))
        .map(|action| {
            action
                .subject
                .clone()
                .unwrap_or_else(|| action.id.replace('_', " "))
        })
        .collect::<Vec<_>>();

    if unsupported.is_empty() {
        return Ok(());
    }

    Err(format!(
        "This version of the app cannot finish these setup steps yet: {}.",
        unsupported.join(", ")
    ))
}

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
        file_name: None,
        expected_hashes: None,
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
        file_name: None,
        expected_hashes: None,
    });

    actions.extend(resource_actions(manifest, &request.profile, installed));

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
            file_name: None,
            expected_hashes: None,
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
        file_name: None,
        expected_hashes: None,
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
        file_name: None,
        expected_hashes: None,
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
            file_name: None,
            expected_hashes: None,
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
                    id: "loader_version".to_string(),
                    kind: SetupActionKind::VerifyLoader,
                    intent: SetupActionIntent::Verify,
                    status: SetupActionStatus::Ready,
                    required: true,
                    resource_id: None,
                    subject: Some(version),
                    target: None,
                    file_name: None,
                    expected_hashes: None,
                },
                SetupActionPreview {
                    id: "fabric_install".to_string(),
                    kind: SetupActionKind::InstallLoader,
                    intent: setup_intent,
                    status: SetupActionStatus::Ready,
                    required: true,
                    resource_id: None,
                    subject: Some(manifest.minecraft.version.clone()),
                    target: None,
                    file_name: None,
                    expected_hashes: None,
                },
            ]
        }
    }
}

fn resource_actions(
    manifest: &SetupManifest,
    profile: &str,
    installed: Option<&InstalledServerSnapshot>,
) -> Vec<SetupActionPreview> {
    let installed_resources = installed_resource_map(installed);
    let selected = selected_resources(manifest, profile);
    let selected_resource_ids = selected
        .iter()
        .map(|resource| resource.id.as_str())
        .collect::<HashSet<_>>();
    let mut actions = selected
        .iter()
        .copied()
        .map(|resource| {
            let installed_resource = installed_resources.get(resource.id.as_str());

            SetupActionPreview {
                id: format!("resource_{}", resource.id),
                kind: SetupActionKind::SyncResource,
                intent: resource_intent(resource, installed_resource),
                status: if can_sync_resource(resource) {
                    SetupActionStatus::Ready
                } else {
                    SetupActionStatus::NotImplemented
                },
                required: resource.required,
                resource_id: Some(resource.id.clone()),
                subject: Some(resource.name.clone()),
                target: Some(setup_action_target(resource.target.clone())),
                file_name: managed_file_name(resource),
                expected_hashes: None,
            }
        })
        .collect::<Vec<_>>();

    actions.extend(selected.into_iter().filter_map(|resource| {
        let installed_resource = installed_resources.get(resource.id.as_str())?;
        if !matches!(
            resource_intent(resource, Some(installed_resource)),
            SetupActionIntent::Update
        ) {
            return None;
        }
        let file_name = installed_resource.file_name.clone()?;

        Some(SetupActionPreview {
            id: format!("remove_replaced_resource_{}", resource.id),
            kind: SetupActionKind::RemoveResource,
            intent: SetupActionIntent::Remove,
            status: SetupActionStatus::Ready,
            required: false,
            resource_id: Some(resource.id.clone()),
            subject: Some(resource.name.clone()),
            target: Some(setup_action_target(installed_resource.target.clone())),
            file_name: Some(file_name),
            expected_hashes: Some(installed_resource.hashes.clone()),
        })
    }));

    if let Some(installed) = installed {
        actions.extend(
            installed
                .resources
                .iter()
                .filter(|resource| !selected_resource_ids.contains(resource.id.as_str()))
                .map(|resource| {
                    let file_name = resource.file_name.clone();

                    SetupActionPreview {
                        id: format!("remove_resource_{}", resource.id),
                        kind: SetupActionKind::RemoveResource,
                        intent: SetupActionIntent::Remove,
                        status: if file_name.is_some() {
                            SetupActionStatus::Ready
                        } else {
                            SetupActionStatus::NotImplemented
                        },
                        required: false,
                        resource_id: Some(resource.id.clone()),
                        subject: Some(resource.name.clone()),
                        target: Some(setup_action_target(resource.target.clone())),
                        file_name,
                        expected_hashes: Some(resource.hashes.clone()),
                    }
                }),
        );
    }

    actions
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
        Some(installed) if resource_matches_installed(resource, installed) => {
            SetupActionIntent::Verify
        }
        Some(_) => SetupActionIntent::Update,
    }
}

fn resource_matches_installed(
    resource: &ManifestResource,
    installed: &InstalledResourceSnapshot,
) -> bool {
    if installed.target != resource.target || installed.source != resource.source {
        return false;
    }
    if resource
        .file_name
        .as_ref()
        .is_some_and(|file_name| installed.file_name.as_ref() != Some(file_name))
    {
        return false;
    }
    if resource
        .hashes
        .sha512
        .as_ref()
        .is_some_and(|hash| installed.hashes.sha512.as_ref() != Some(hash))
    {
        return false;
    }
    if resource
        .hashes
        .sha256
        .as_ref()
        .is_some_and(|hash| installed.hashes.sha256.as_ref() != Some(hash))
    {
        return false;
    }

    true
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
    fn unsupported_actions_block_the_plan_before_installation() {
        let mut plan = install_plan(vec![]);
        plan.actions = vec![SetupActionPreview {
            id: "fabric_install".to_string(),
            kind: SetupActionKind::InstallLoader,
            intent: SetupActionIntent::Add,
            status: SetupActionStatus::NotImplemented,
            required: true,
            resource_id: None,
            subject: Some("Fabric".to_string()),
            target: None,
            file_name: None,
            expected_hashes: None,
        }];

        let error = ensure_plan_is_supported(&plan).expect_err("unsupported plan must fail");

        assert!(error.contains("Fabric"));
    }

    #[test]
    fn fully_supported_actions_allow_the_plan() {
        let mut plan = install_plan(vec![]);
        plan.actions = vec![SetupActionPreview {
            id: "game_directory".to_string(),
            kind: SetupActionKind::EnsureGameDirectory,
            intent: SetupActionIntent::Add,
            status: SetupActionStatus::Ready,
            required: true,
            resource_id: None,
            subject: Some("Example".to_string()),
            target: None,
            file_name: None,
            expected_hashes: None,
        }];

        assert!(ensure_plan_is_supported(&plan).is_ok());
    }

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
        assert!(actions.iter().any(|action| {
            action.kind == SetupActionKind::RemoveResource
                && action.resource_id.as_deref() == Some("fabric-api")
                && action.expected_hashes == Some(hashes("old"))
        }));
    }

    #[test]
    fn modrinth_api_hashes_do_not_create_permanent_updates() {
        let manifest = manifest_with_resource(resource(
            "fabric-api",
            ManifestResourceSource::Modrinth {
                project: "project-id".to_string(),
                version: "version-id".to_string(),
            },
            ManifestResourceHashes::default(),
        ));
        let installed = installed_snapshot(vec![installed_resource(
            "fabric-api",
            ManifestResourceSource::Modrinth {
                project: "project-id".to_string(),
                version: "version-id".to_string(),
            },
            ManifestResourceHashes {
                sha512: Some("resolved".to_string()),
                sha256: None,
            },
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

    #[test]
    fn resource_actions_mark_stale_installed_resources_as_remove() {
        let manifest = manifest_with_resource(resource(
            "fabric-api",
            ManifestResourceSource::Modrinth {
                project: "fabric-api".to_string(),
                version: "1.0.0".to_string(),
            },
            hashes("abc"),
        ));
        let installed = installed_snapshot(vec![installed_resource(
            "old-map",
            ManifestResourceSource::Direct {
                url: "https://example.com/old-map.zip".to_string(),
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
            Some((
                SetupActionKind::RemoveResource,
                SetupActionIntent::Remove,
                SetupActionStatus::Ready
            )),
            resource_action(&actions, "old-map").map(|action| (
                action.kind,
                action.intent,
                action.status
            ))
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
            manifest_fingerprint: "sha256:example".to_string(),
            launcher: LauncherKind::Official,
            profile: "balanced".to_string(),
            server_address: "play.example.com".to_string(),
        }
    }

    fn install_plan(actions: Vec<SetupActionPreview>) -> crate::commands::InstallPlan {
        crate::commands::InstallPlan {
            server_id: "example".to_string(),
            update_status: ServerUpdateStatus::NewSetup,
            minecraft_version: "1.21.6".to_string(),
            loader_kind: ManifestLoaderKind::Fabric,
            loader_version: Some("0.16.14".to_string()),
            game_directory_name: "Example".to_string(),
            server_name: "Example".to_string(),
            server_address: "play.example.com".to_string(),
            launcher: LauncherKind::Official,
            profile: "balanced".to_string(),
            profile_label: "Balanced".to_string(),
            recommended_memory_mb: 4096,
            actions,
            required_mods: vec![],
            optional_mods: vec![],
            warnings: vec![],
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
            profiles: vec![],
            file_name: Some(format!("{id}.jar")),
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
            name: id.to_string(),
            target: ManifestResourceTarget::Mods,
            file_name: Some(format!("{id}.jar")),
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
