use std::path::Path;

use crate::commands::{InstallPlan, LauncherDetection, LauncherDetectionStatus, LauncherKind};
use crate::system::paths;

use super::{
    minecraft_profiles, LauncherAdapter, LauncherProfileResult, LauncherProfileValidation,
};

pub struct OfficialMinecraftLauncherAdapter;
const LAUNCHER_NAME: &str = "Minecraft Launcher";

impl LauncherAdapter for OfficialMinecraftLauncherAdapter {
    fn detection(&self) -> LauncherDetection {
        let Ok(path) = paths::default_minecraft_dir() else {
            return LauncherDetection {
                kind: LauncherKind::Official,
                status: LauncherDetectionStatus::NotFound,
                setup_supported: true,
                detail: "Could not find the Minecraft folder.".to_string(),
                confidence: 0.4,
            };
        };

        let profiles_path = path.join("launcher_profiles.json");

        if profiles_path.is_file() {
            LauncherDetection {
                kind: LauncherKind::Official,
                status: LauncherDetectionStatus::Detected,
                setup_supported: true,
                detail: format!("Launcher profiles found at {}.", profiles_path.display()),
                confidence: 0.95,
            }
        } else if path.is_dir() {
            LauncherDetection {
                kind: LauncherKind::Official,
                status: LauncherDetectionStatus::Detected,
                setup_supported: true,
                detail: format!(
                    "Minecraft folder found at {}, but launcher profiles were not found.",
                    path.display()
                ),
                confidence: 0.65,
            }
        } else {
            LauncherDetection {
                kind: LauncherKind::Official,
                status: LauncherDetectionStatus::NotFound,
                setup_supported: true,
                detail: format!("Minecraft folder was not found at {}.", path.display()),
                confidence: 0.7,
            }
        }
    }
}

impl OfficialMinecraftLauncherAdapter {
    pub fn ensure_profile(
        &self,
        plan: &InstallPlan,
        game_dir: &Path,
    ) -> Result<LauncherProfileResult, String> {
        minecraft_profiles::ensure_profile(plan, game_dir, &plan.server_id, LAUNCHER_NAME)
    }

    pub fn validate_profile(
        &self,
        plan: &InstallPlan,
        game_dir: &Path,
    ) -> Result<LauncherProfileValidation, String> {
        minecraft_profiles::validate_profile(plan, game_dir, &plan.server_id)
    }
}

pub fn validate_profile_prerequisites(plan: &InstallPlan) -> Result<(), String> {
    minecraft_profiles::validate_profile_prerequisites(plan, LAUNCHER_NAME)
}

pub fn validate_base_prerequisites(plan: &InstallPlan) -> Result<(), String> {
    minecraft_profiles::validate_base_prerequisites(plan, LAUNCHER_NAME)
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    use serde_json::json;

    use super::*;
    use crate::commands::{InstallPlan, LauncherKind, ServerUpdateStatus};
    use crate::launcher::LauncherProfileAction;
    use crate::manifest::schema::ManifestLoaderKind;
    use crate::minecraft::version;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn rejects_unfamiliar_launcher_profile_container_shapes() {
        let mut non_object_root = json!([]);
        let mut non_object_profiles = json!({ "profiles": [] });

        assert!(minecraft_profiles::profiles_object_mut(&mut non_object_root).is_err());
        assert!(minecraft_profiles::profiles_object_mut(&mut non_object_profiles).is_err());
        assert_eq!(non_object_root, json!([]));
        assert_eq!(non_object_profiles, json!({ "profiles": [] }));
    }

    #[test]
    fn vanilla_profiles_use_the_minecraft_version_directly() {
        let mut plan = install_plan();
        plan.loader_kind = ManifestLoaderKind::None;
        plan.loader_version = None;

        assert_eq!(version::installed_version_id(&plan), "1.21.6");
        assert_eq!(version::version_label(&plan), "Minecraft 1.21.6");
    }

    #[test]
    fn ensure_profile_preserves_unknown_fields_and_is_idempotent() {
        let _guard = ENV_LOCK.lock().expect("test env lock poisoned");
        let home = test_home("ensure-profile");
        let _environment = TestEnvironment::new(&home);
        let minecraft_dir = paths::default_minecraft_dir().expect("resolve Minecraft dir");
        let profiles_path = minecraft_dir.join("launcher_profiles.json");
        let version_id = "fabric-loader-0.16.14-1.21.6";
        let version_path = minecraft_dir
            .join("versions")
            .join(version_id)
            .join(format!("{version_id}.json"));
        std::fs::create_dir_all(version_path.parent().expect("version parent"))
            .expect("create version parent");
        std::fs::write(&version_path, "{}").expect("write version");
        std::fs::write(
            &profiles_path,
            serde_json::to_string_pretty(&json!({
                "profiles": {
                    "other-profile": {
                        "name": "Other",
                        "type": "custom"
                    },
                    "example-server": {
                        "created": "2026-01-01T00:00:00.000Z",
                        "gameDir": "/old/path",
                        "icon": "Crafting_Table",
                        "lastUsed": "2026-01-02T00:00:00.000Z",
                        "lastVersionId": "old-version",
                        "name": "Old Example Server",
                        "type": "custom",
                        "unknownProfileField": true
                    }
                },
                "settings": {
                    "unknownSetting": true
                },
                "version": 4
            }))
            .expect("serialize profiles"),
        )
        .expect("write profiles");

        let plan = install_plan();
        let game_dir = home.join("game-dir");
        let adapter = OfficialMinecraftLauncherAdapter;
        let result = adapter
            .ensure_profile(&plan, &game_dir)
            .expect("ensure profile");

        assert!(matches!(result.action, LauncherProfileAction::Updated));
        assert!(result
            .backup_path
            .as_ref()
            .is_some_and(|path| path.is_file()));

        let written = read_json(&profiles_path);
        assert_eq!(written["settings"]["unknownSetting"], true);
        assert_eq!(written["profiles"]["other-profile"]["name"], "Other");
        assert_eq!(
            written["profiles"]["example-server"]["unknownProfileField"],
            true
        );
        assert_eq!(
            written["profiles"]["example-server"]["icon"],
            "Crafting_Table"
        );
        assert_eq!(
            written["profiles"]["example-server"]["gameDir"],
            game_dir.display().to_string()
        );
        assert_eq!(
            written["profiles"]["example-server"]["lastVersionId"],
            version_id
        );
        assert_eq!(
            written["profiles"]["example-server"]["name"],
            "Custom Launcher Profile"
        );

        let second_result = adapter
            .ensure_profile(&plan, &game_dir)
            .expect("ensure profile again");
        assert!(matches!(
            second_result.action,
            LauncherProfileAction::Unchanged
        ));
        assert!(second_result.backup_path.is_none());
    }

    #[test]
    fn ensure_profile_fails_before_write_when_fabric_version_is_missing() {
        let _guard = ENV_LOCK.lock().expect("test env lock poisoned");
        let home = test_home("missing-fabric");
        let _environment = TestEnvironment::new(&home);
        let minecraft_dir = paths::default_minecraft_dir().expect("resolve Minecraft dir");
        let profiles_path = minecraft_dir.join("launcher_profiles.json");
        std::fs::create_dir_all(&minecraft_dir).expect("create minecraft dir");
        std::fs::write(
            &profiles_path,
            serde_json::to_string_pretty(&json!({
                "profiles": {},
                "version": 4
            }))
            .expect("serialize profiles"),
        )
        .expect("write profiles");
        let before = std::fs::read_to_string(&profiles_path).expect("read before");

        let adapter = OfficialMinecraftLauncherAdapter;
        let error = adapter
            .ensure_profile(&install_plan(), &home.join("game-dir"))
            .expect_err("expected missing fabric error");

        assert!(error.contains("Fabric 0.16.14 for Minecraft 1.21.6 is not installed"));
        let after = std::fs::read_to_string(&profiles_path).expect("read after");
        assert_eq!(before, after);
        assert!(std::fs::read_dir(&minecraft_dir)
            .expect("read minecraft dir")
            .filter_map(Result::ok)
            .all(|entry| !entry.file_name().to_string_lossy().contains(".bak")));
    }

    fn install_plan() -> InstallPlan {
        InstallPlan {
            server_id: "example-server".to_string(),
            minecraft_version: "1.21.6".to_string(),
            loader_kind: ManifestLoaderKind::Fabric,
            loader_version: Some("0.16.14".to_string()),
            game_directory_name: "Example Server".to_string(),
            server_name: "Example Server".to_string(),
            server_address: "play.example.com".to_string(),
            launcher_profile_name: "Custom Launcher Profile".to_string(),
            launcher: LauncherKind::Official,
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

    fn test_home(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_nanos();
        let path = env::temp_dir().join(format!("minecraft-setup-manager-{name}-{unique}"));
        std::fs::create_dir_all(&path).expect("create test home");
        path
    }

    fn read_json(path: &Path) -> serde_json::Value {
        serde_json::from_str(&std::fs::read_to_string(path).expect("read json"))
            .expect("parse json")
    }

    struct TestEnvironment {
        previous: Vec<(&'static str, Option<std::ffi::OsString>)>,
    }

    impl TestEnvironment {
        fn new(home: &Path) -> Self {
            let values = [
                ("HOME", home.to_path_buf()),
                ("USERPROFILE", home.to_path_buf()),
                ("APPDATA", home.join("AppData").join("Roaming")),
                ("XDG_DATA_HOME", home.join(".local").join("share")),
            ];
            let previous = values
                .iter()
                .map(|(name, _)| (*name, env::var_os(name)))
                .collect();

            for (name, value) in values {
                unsafe {
                    env::set_var(name, value);
                }
            }

            Self { previous }
        }
    }

    impl Drop for TestEnvironment {
        fn drop(&mut self) {
            for (name, value) in &self.previous {
                unsafe {
                    if let Some(value) = value {
                        env::set_var(name, value);
                    } else {
                        env::remove_var(name);
                    }
                }
            }
        }
    }
}
