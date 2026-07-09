use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::commands::{InstallPlan, SetupActionKind, SetupActionTarget};
use crate::manifest::schema::{ManifestResource, ManifestResourceSource, ManifestResourceTarget};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagedResourceAction {
    Removed,
    Missing,
    SkippedNoFileName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedResourceResult {
    pub resource_id: String,
    pub action: ManagedResourceAction,
    pub path: Option<PathBuf>,
}

pub fn apply_plan_resource_actions(
    plan: &InstallPlan,
    game_dir: &Path,
) -> Result<Vec<ManagedResourceResult>, String> {
    plan.actions
        .iter()
        .filter(|action| matches!(action.kind, SetupActionKind::RemoveResource))
        .map(|action| {
            let resource_id = action.resource_id.clone().ok_or_else(|| {
                "A managed resource removal is missing its resource id.".to_string()
            })?;
            let Some(file_name) = &action.file_name else {
                return Ok(ManagedResourceResult {
                    resource_id,
                    action: ManagedResourceAction::SkippedNoFileName,
                    path: None,
                });
            };
            let target = action
                .target
                .ok_or_else(|| format!("Managed resource {resource_id} is missing its target."))?;
            let path = managed_file_path(game_dir, target, file_name)?;

            if !path.is_file() {
                return Ok(ManagedResourceResult {
                    resource_id,
                    action: ManagedResourceAction::Missing,
                    path: Some(path),
                });
            }

            fs::remove_file(&path).map_err(|error| {
                format!(
                    "Could not remove managed resource {} at {}: {error}",
                    resource_id,
                    path.display()
                )
            })?;

            Ok(ManagedResourceResult {
                resource_id,
                action: ManagedResourceAction::Removed,
                path: Some(path),
            })
        })
        .collect()
}

pub fn managed_file_name(resource: &ManifestResource) -> Option<String> {
    resource
        .file_name
        .clone()
        .or_else(|| direct_source_file_name(&resource.source))
        .filter(|file_name| is_safe_file_name(file_name))
}

fn direct_source_file_name(source: &ManifestResourceSource) -> Option<String> {
    let ManifestResourceSource::Direct { url } = source else {
        return None;
    };

    let parsed = reqwest::Url::parse(url).ok()?;
    parsed
        .path_segments()
        .and_then(Iterator::last)
        .filter(|segment| !segment.is_empty())
        .map(ToString::to_string)
}

fn managed_file_path(
    game_dir: &Path,
    target: SetupActionTarget,
    file_name: &str,
) -> Result<PathBuf, String> {
    if !is_safe_file_name(file_name) {
        return Err(format!(
            "Managed resource file name {file_name:?} must be a plain file name."
        ));
    }

    Ok(game_dir.join(target_dir_name(target)).join(file_name))
}

fn is_safe_file_name(file_name: &str) -> bool {
    !file_name.trim().is_empty()
        && !file_name.contains('/')
        && !file_name.contains('\\')
        && Path::new(file_name)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn target_dir_name(target: SetupActionTarget) -> &'static str {
    match target {
        SetupActionTarget::Mods => "mods",
        SetupActionTarget::Resourcepacks => "resourcepacks",
        SetupActionTarget::Shaderpacks => "shaderpacks",
        SetupActionTarget::Config => "config",
    }
}

pub fn setup_action_target(target: ManifestResourceTarget) -> SetupActionTarget {
    match target {
        ManifestResourceTarget::Mods => SetupActionTarget::Mods,
        ManifestResourceTarget::Resourcepacks => SetupActionTarget::Resourcepacks,
        ManifestResourceTarget::Shaderpacks => SetupActionTarget::Shaderpacks,
        ManifestResourceTarget::Config => SetupActionTarget::Config,
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::commands::{
        InstallPlan, LauncherKind, PerformanceProfileId, ServerUpdateStatus, SetupActionIntent,
        SetupActionPreview, SetupActionStatus,
    };

    use super::*;

    #[test]
    fn applies_stale_resource_removal_inside_managed_target_dir() {
        let game_dir = test_dir("remove-managed-resource");
        let mods_dir = game_dir.join("mods");
        fs::create_dir_all(&mods_dir).expect("create mods dir");
        let file_path = mods_dir.join("old.jar");
        fs::write(&file_path, "old").expect("write managed file");

        let results = apply_plan_resource_actions(&plan_with_removal(Some("old.jar")), &game_dir)
            .expect("apply resource actions");

        assert!(!file_path.exists());
        assert_eq!(
            results,
            vec![ManagedResourceResult {
                resource_id: "old-mod".to_string(),
                action: ManagedResourceAction::Removed,
                path: Some(file_path)
            }]
        );
    }

    #[test]
    fn rejects_path_like_resource_file_names() {
        let game_dir = test_dir("reject-path-resource");

        let error =
            apply_plan_resource_actions(&plan_with_removal(Some("../outside.jar")), &game_dir)
                .expect_err("expected unsafe path error");

        assert!(error.contains("plain file name"));
    }

    #[test]
    fn rejects_windows_style_path_like_resource_file_names() {
        let game_dir = test_dir("reject-windows-path-resource");

        let error =
            apply_plan_resource_actions(&plan_with_removal(Some("folder\\outside.jar")), &game_dir)
                .expect_err("expected unsafe path error");

        assert!(error.contains("plain file name"));
    }

    #[test]
    fn skips_resource_removal_without_file_name() {
        let game_dir = test_dir("skip-missing-resource-file-name");

        let results = apply_plan_resource_actions(&plan_with_removal(None), &game_dir)
            .expect("apply resource actions");

        assert_eq!(
            results,
            vec![ManagedResourceResult {
                resource_id: "old-mod".to_string(),
                action: ManagedResourceAction::SkippedNoFileName,
                path: None
            }]
        );
    }

    fn plan_with_removal(file_name: Option<&str>) -> InstallPlan {
        InstallPlan {
            server_id: "example".to_string(),
            update_status: ServerUpdateStatus::UpdateAvailable,
            minecraft_version: "1.21.6".to_string(),
            fabric_loader_version: "0.16.14".to_string(),
            game_directory_name: "Example".to_string(),
            server_name: "Example".to_string(),
            server_address: "play.example.com".to_string(),
            launcher: LauncherKind::Official,
            profile: PerformanceProfileId::Balanced,
            actions: vec![SetupActionPreview {
                id: "remove_resource_old-mod".to_string(),
                kind: SetupActionKind::RemoveResource,
                intent: SetupActionIntent::Remove,
                status: SetupActionStatus::NotImplemented,
                required: false,
                resource_id: Some("old-mod".to_string()),
                subject: Some("Old Mod".to_string()),
                target: Some(SetupActionTarget::Mods),
                file_name: file_name.map(str::to_string),
            }],
            required_mods: vec![],
            optional_mods: vec![],
            warnings: vec![],
        }
    }

    fn test_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_nanos();
        let path = env::temp_dir().join(format!("minecraft-setup-manager-{name}-{unique}"));
        fs::create_dir_all(&path).expect("create test dir");
        path
    }
}
