use crate::commands::{
    InstallPlanRequest, LauncherKind, PerformanceProfileId, ServerUpdateStatus, SetupActionIntent,
    SetupActionKind, SetupActionPreview, SetupActionStatus, SetupActionTarget,
};
use crate::manifest::schema::{ManifestLoaderKind, ManifestResourceTarget, SetupManifest};
use crate::manifest::selected_resources;

pub fn build_action_previews(
    manifest: &SetupManifest,
    request: &InstallPlanRequest,
    update_status: ServerUpdateStatus,
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

    actions.extend(resource_actions(manifest, request.profile, setup_intent));

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
    setup_intent: SetupActionIntent,
) -> Vec<SetupActionPreview> {
    selected_resources(manifest, profile)
        .into_iter()
        .map(|resource| SetupActionPreview {
            id: format!("resource_{}", resource.id),
            kind: SetupActionKind::SyncResource,
            intent: setup_intent,
            status: SetupActionStatus::NotImplemented,
            required: resource.required,
            resource_id: Some(resource.id.clone()),
            subject: Some(resource.name.clone()),
            target: Some(setup_action_target(resource.target.clone())),
        })
        .collect()
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
