use std::path::Path;

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::commands::{InstallPlan, LauncherDetection, LauncherDetectionStatus, LauncherKind};
use crate::system::paths;

use super::{
    minecraft_profiles, LauncherAdapter, LauncherProfileResult, LauncherProfileValidation,
};

pub struct SklauncherAdapter;
const LAUNCHER_NAME: &str = "SKlauncher";
const MAX_COMPATIBLE_PROFILE_VERSION: u64 = 6;

impl LauncherAdapter for SklauncherAdapter {
    fn detection(&self) -> LauncherDetection {
        let Ok(minecraft_dir) = paths::default_minecraft_dir() else {
            return not_found_detection("Could not find the Minecraft folder.");
        };

        detection_at(&minecraft_dir)
    }
}

impl SklauncherAdapter {
    pub fn ensure_profile(
        &self,
        plan: &InstallPlan,
        game_dir: &Path,
    ) -> Result<LauncherProfileResult, String> {
        validate_profile_prerequisites(plan)?;
        let profile_id = profile_id(plan);
        minecraft_profiles::ensure_profile(plan, game_dir, &profile_id, LAUNCHER_NAME)
    }

    pub fn validate_profile(
        &self,
        plan: &InstallPlan,
        game_dir: &Path,
    ) -> Result<LauncherProfileValidation, String> {
        validate_sklauncher_profile_format()?;
        minecraft_profiles::validate_profile(plan, game_dir, &profile_id(plan))
    }
}

pub fn validate_profile_prerequisites(plan: &InstallPlan) -> Result<(), String> {
    validate_sklauncher_profile_format()?;
    minecraft_profiles::validate_profile_prerequisites(plan, LAUNCHER_NAME)
}

pub fn validate_base_prerequisites(plan: &InstallPlan) -> Result<(), String> {
    validate_sklauncher_profile_format()?;
    minecraft_profiles::validate_base_prerequisites(plan, LAUNCHER_NAME)
}

fn detection_at(minecraft_dir: &Path) -> LauncherDetection {
    let data_dir = minecraft_dir.join("sklauncher");
    let log_path = data_dir.join("sklauncher_logs.txt");

    if log_path.is_file() {
        LauncherDetection {
            kind: LauncherKind::Sklauncher,
            status: LauncherDetectionStatus::Detected,
            setup_supported: true,
            detail: format!(
                "Compatible SKlauncher data found at {}.",
                data_dir.display()
            ),
            confidence: 0.95,
        }
    } else if data_dir.is_dir() {
        LauncherDetection {
            kind: LauncherKind::Sklauncher,
            status: LauncherDetectionStatus::Detected,
            setup_supported: true,
            detail: format!("SKlauncher data found at {}.", data_dir.display()),
            confidence: 0.8,
        }
    } else {
        not_found_detection(&format!(
            "SKlauncher data was not found at {}.",
            data_dir.display()
        ))
    }
}

fn not_found_detection(detail: &str) -> LauncherDetection {
    LauncherDetection {
        kind: LauncherKind::Sklauncher,
        status: LauncherDetectionStatus::NotFound,
        setup_supported: false,
        detail: detail.to_string(),
        confidence: 0.8,
    }
}

fn validate_sklauncher_profile_format() -> Result<(), String> {
    let path = paths::minecraft_launcher_profiles_file()?;
    validate_sklauncher_profile_format_at(&path)
}

fn validate_sklauncher_profile_format_at(path: &Path) -> Result<(), String> {
    let root = minecraft_profiles::read_launcher_profiles(path)?;
    let version = root.get("version").and_then(Value::as_u64);
    let profiles_are_valid = root.get("profiles").is_some_and(Value::is_object);

    if profiles_are_valid
        && version.is_some_and(|version| (1..=MAX_COMPATIBLE_PROFILE_VERSION).contains(&version))
    {
        return Ok(());
    }

    Err(format!(
        "SKlauncher uses an unsupported profile format at {}. This version supports stable SKlauncher 3.2 with profile formats 1 through {MAX_COMPATIBLE_PROFILE_VERSION}.",
        path.display()
    ))
}

fn profile_id(plan: &InstallPlan) -> String {
    Sha256::digest(plan.server_id.as_bytes())[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::commands::{InstallPlan, ServerUpdateStatus};
    use crate::manifest::schema::ManifestLoaderKind;

    #[test]
    fn detects_only_the_stable_sklauncher_workspace_markers() {
        let minecraft_dir = test_dir("detection");
        let missing = detection_at(&minecraft_dir);
        assert!(matches!(missing.status, LauncherDetectionStatus::NotFound));
        assert!(!missing.setup_supported);

        let data_dir = minecraft_dir.join("sklauncher");
        std::fs::create_dir_all(&data_dir).expect("create SKlauncher data dir");
        let directory_detection = detection_at(&minecraft_dir);
        assert!(matches!(
            directory_detection.status,
            LauncherDetectionStatus::Detected
        ));
        assert!(directory_detection.setup_supported);
        assert_eq!(directory_detection.confidence, 0.8);

        std::fs::write(data_dir.join("sklauncher_logs.txt"), "stable 3.2")
            .expect("write launcher log");
        let log_detection = detection_at(&minecraft_dir);
        assert!(matches!(
            log_detection.status,
            LauncherDetectionStatus::Detected
        ));
        assert!(log_detection.setup_supported);
        assert_eq!(log_detection.confidence, 0.95);
    }

    #[test]
    fn profile_ids_are_stable_compact_uuid_keys() {
        let first = profile_id(&install_plan("server-a"));
        let same = profile_id(&install_plan("server-a"));
        let second = profile_id(&install_plan("server-b"));

        assert_eq!(first, same);
        assert_ne!(first, second);
        assert_eq!(first.len(), 32);
        assert!(first.chars().all(|character| character.is_ascii_hexdigit()));
    }

    #[test]
    fn accepts_only_the_verified_sklauncher_profile_formats() {
        let directory = test_dir("profile-format");
        std::fs::create_dir_all(&directory).expect("create profile fixture directory");
        let path = directory.join("launcher_profiles.json");

        std::fs::write(&path, r#"{"profiles":{},"version":6}"#)
            .expect("write supported profile fixture");
        assert!(validate_sklauncher_profile_format_at(&path).is_ok());

        std::fs::write(&path, r#"{"profiles":{},"version":7}"#)
            .expect("write future profile fixture");
        assert!(validate_sklauncher_profile_format_at(&path).is_err());

        std::fs::write(&path, r#"{"profiles":[],"version":6}"#)
            .expect("write invalid profile fixture");
        assert!(validate_sklauncher_profile_format_at(&path).is_err());
    }

    fn install_plan(server_id: &str) -> InstallPlan {
        InstallPlan {
            server_id: server_id.to_string(),
            minecraft_version: "1.21.6".to_string(),
            loader_kind: ManifestLoaderKind::Fabric,
            loader_version: Some("0.16.14".to_string()),
            game_directory_name: "Example Server".to_string(),
            server_name: "Example Server".to_string(),
            server_address: "play.example.com".to_string(),
            launcher_profile_name: "Custom Launcher Profile".to_string(),
            launcher: LauncherKind::Sklauncher,
            profile: "balanced".to_string(),
            profile_label: "Balanced".to_string(),
            recommended_memory_mb: 4096,
            update_status: ServerUpdateStatus::NewSetup,
            actions: vec![],
            resources: vec![],
            required_mods: vec![],
            optional_mods: vec![],
            warnings: vec![],
        }
    }

    fn test_dir(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("minecraft-setup-manager-{name}-{unique}"))
    }
}
