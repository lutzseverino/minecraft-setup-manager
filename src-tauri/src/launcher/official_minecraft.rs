use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde_json::{Map, Value};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::commands::{InstallPlan, LauncherDetection, LauncherDetectionStatus, LauncherKind};
use crate::manifest::schema::ManifestLoaderKind;
use crate::system::paths;
use crate::system::{atomic_file, path_safety};

use super::{
    LauncherAdapter, LauncherProfileAction, LauncherProfileResult, LauncherProfileValidation,
};

pub struct OfficialMinecraftLauncherAdapter;
static PROFILE_WRITE_LOCK: Mutex<()> = Mutex::new(());

impl LauncherAdapter for OfficialMinecraftLauncherAdapter {
    fn detection(&self) -> LauncherDetection {
        let Ok(path) = paths::default_minecraft_dir() else {
            return LauncherDetection {
                kind: LauncherKind::Official,
                status: LauncherDetectionStatus::NotFound,
                detail: "Could not find the Minecraft folder.".to_string(),
                confidence: 0.4,
            };
        };

        let profiles_path = path.join("launcher_profiles.json");

        if profiles_path.is_file() {
            LauncherDetection {
                kind: LauncherKind::Official,
                status: LauncherDetectionStatus::Detected,
                detail: format!("Launcher profiles found at {}.", profiles_path.display()),
                confidence: 0.95,
            }
        } else if path.is_dir() {
            LauncherDetection {
                kind: LauncherKind::Official,
                status: LauncherDetectionStatus::Detected,
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
        let _guard = PROFILE_WRITE_LOCK
            .lock()
            .map_err(|_| "The Minecraft Launcher profile lock is unavailable.".to_string())?;
        validate_profile_prerequisites(plan)?;

        let launcher_profiles_path = paths::minecraft_launcher_profiles_file()?;
        let mut root = read_launcher_profiles(&launcher_profiles_path)?;
        let profiles = profiles_object_mut(&mut root)?;
        let profile_id = profile_id(plan);
        let version_id = version_id(plan);
        let expected_game_dir = game_dir.display().to_string();
        let existing_profile = profiles.get(&profile_id).cloned();
        let mut next_profile = existing_profile
            .as_ref()
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();

        let created = next_profile
            .get("created")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(current_timestamp);
        let icon = next_profile
            .get("icon")
            .and_then(Value::as_str)
            .unwrap_or("Grass")
            .to_string();

        next_profile.insert("created".to_string(), Value::String(created));
        next_profile.insert(
            "gameDir".to_string(),
            Value::String(expected_game_dir.clone()),
        );
        next_profile.insert("icon".to_string(), Value::String(icon));
        next_profile.insert(
            "lastVersionId".to_string(),
            Value::String(version_id.clone()),
        );
        next_profile.insert("name".to_string(), Value::String(plan.server_name.clone()));
        next_profile.insert("type".to_string(), Value::String("custom".to_string()));

        let next_value = Value::Object(next_profile);
        let action = match existing_profile {
            Some(current) if current == next_value => LauncherProfileAction::Unchanged,
            Some(_) => LauncherProfileAction::Updated,
            None => LauncherProfileAction::Created,
        };
        let backup_path = if matches!(action, LauncherProfileAction::Unchanged) {
            None
        } else {
            Some(backup_launcher_profiles(&launcher_profiles_path)?)
        };

        profiles.insert(profile_id.clone(), next_value);

        if !matches!(action, LauncherProfileAction::Unchanged) {
            write_launcher_profiles(&launcher_profiles_path, &root)?;
        }

        Ok(LauncherProfileResult {
            profile_id,
            launcher_profiles_path: Some(launcher_profiles_path),
            backup_path,
            game_dir: game_dir.to_path_buf(),
            version_id,
            action: action.clone(),
            log: vec![profile_log_line(&action)],
        })
    }

    pub fn validate_profile(
        &self,
        plan: &InstallPlan,
        game_dir: &Path,
    ) -> Result<LauncherProfileValidation, String> {
        let launcher_profiles_path = paths::minecraft_launcher_profiles_file()?;
        let version_id = version_id(plan);
        let version_exists = paths::minecraft_version_file(&version_id)?.is_file();
        let launcher_profiles_exists = launcher_profiles_path.is_file();
        let root = if launcher_profiles_exists {
            read_launcher_profiles(&launcher_profiles_path).ok()
        } else {
            None
        };
        let profile = root
            .as_ref()
            .and_then(|value| value.get("profiles"))
            .and_then(Value::as_object)
            .and_then(|profiles| profiles.get(&profile_id(plan)));
        let expected_game_dir = game_dir.display().to_string();
        let profile_exists = profile.is_some();
        let game_dir_matches = profile
            .and_then(|value| value.get("gameDir"))
            .and_then(Value::as_str)
            == Some(expected_game_dir.as_str());
        let version_matches = profile
            .and_then(|value| value.get("lastVersionId"))
            .and_then(Value::as_str)
            == Some(version_id.as_str());

        Ok(LauncherProfileValidation {
            required: true,
            launcher_profiles_path: Some(launcher_profiles_path),
            launcher_profiles_exists,
            version_exists,
            profile_exists,
            game_dir_matches,
            version_matches,
            profile_id: profile_id(plan),
            expected_game_dir: game_dir.to_path_buf(),
            expected_version_id: version_id,
        })
    }
}

pub fn validate_profile_prerequisites(plan: &InstallPlan) -> Result<(), String> {
    let launcher_profiles_path = paths::minecraft_launcher_profiles_file()?;
    if !launcher_profiles_path.is_file() {
        return Err(format!(
            "Could not find Minecraft Launcher profiles at {}. Open Minecraft Launcher once, then try again.",
            launcher_profiles_path.display()
        ));
    }

    let version_id = version_id(plan);
    let version_path = paths::minecraft_version_file(&version_id)?;
    if !version_path.is_file() {
        return Err(format!(
            "{} is not installed yet. Open Minecraft Launcher, install or run that version once, then try again.",
            version_label(plan)
        ));
    }

    Ok(())
}

pub fn version_id(plan: &InstallPlan) -> String {
    match plan.loader_kind {
        ManifestLoaderKind::None => plan.minecraft_version.clone(),
        ManifestLoaderKind::Fabric => format!(
            "fabric-loader-{}-{}",
            plan.loader_version.as_deref().unwrap_or("unknown"),
            plan.minecraft_version
        ),
    }
}

fn version_label(plan: &InstallPlan) -> String {
    match plan.loader_kind {
        ManifestLoaderKind::None => format!("Minecraft {}", plan.minecraft_version),
        ManifestLoaderKind::Fabric => format!(
            "Fabric {} for Minecraft {}",
            plan.loader_version.as_deref().unwrap_or("unknown"),
            plan.minecraft_version
        ),
    }
}

fn profile_id(plan: &InstallPlan) -> String {
    plan.server_id.clone()
}

fn read_launcher_profiles(path: &Path) -> Result<Value, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "Could not read Minecraft Launcher profiles at {}: {error}",
            path.display()
        )
    })?;

    serde_json::from_str(&contents).map_err(|error| {
        format!(
            "Could not parse Minecraft Launcher profiles at {}: {error}",
            path.display()
        )
    })
}

fn profiles_object_mut(root: &mut Value) -> Result<&mut Map<String, Value>, String> {
    let object = root
        .as_object_mut()
        .ok_or_else(|| "Minecraft Launcher profiles file is not a JSON object.".to_string())?;
    let profiles = object
        .entry("profiles")
        .or_insert_with(|| Value::Object(Map::new()));

    profiles
        .as_object_mut()
        .ok_or_else(|| "Minecraft Launcher profiles entry is not a JSON object.".to_string())
}

fn backup_launcher_profiles(path: &Path) -> Result<PathBuf, String> {
    path_safety::reject_symlink(path, "Minecraft Launcher profiles file")?;
    let backup_path = path.with_file_name(format!(
        "launcher_profiles.minecraft-setup-manager-{}.json.bak",
        backup_timestamp()
    ));

    fs::copy(path, &backup_path).map_err(|error| {
        format!(
            "Could not back up Minecraft Launcher profiles to {}: {error}",
            backup_path.display()
        )
    })?;

    Ok(backup_path)
}

fn write_launcher_profiles(path: &Path, root: &Value) -> Result<(), String> {
    let json = serde_json::to_string_pretty(root)
        .map_err(|error| format!("Could not prepare Minecraft Launcher profiles: {error}"))?;
    atomic_file::write(
        path,
        format!("{json}\n").as_bytes(),
        "Minecraft Launcher profiles file",
    )
}

fn profile_log_line(action: &LauncherProfileAction) -> String {
    match action {
        LauncherProfileAction::Created => "Created the Minecraft Launcher profile.".to_string(),
        LauncherProfileAction::Updated => "Updated the Minecraft Launcher profile.".to_string(),
        LauncherProfileAction::Unchanged => {
            "Minecraft Launcher profile was already ready.".to_string()
        }
        LauncherProfileAction::Skipped => "Launcher profile changes were skipped.".to_string(),
    }
}

fn current_timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn backup_timestamp() -> String {
    current_timestamp().replace([':', '-', '.', 'Z'], "")
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

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn rejects_unfamiliar_launcher_profile_container_shapes() {
        let mut non_object_root = json!([]);
        let mut non_object_profiles = json!({ "profiles": [] });

        assert!(profiles_object_mut(&mut non_object_root).is_err());
        assert!(profiles_object_mut(&mut non_object_profiles).is_err());
        assert_eq!(non_object_root, json!([]));
        assert_eq!(non_object_profiles, json!({ "profiles": [] }));
    }

    #[test]
    fn vanilla_profiles_use_the_minecraft_version_directly() {
        let mut plan = install_plan();
        plan.loader_kind = ManifestLoaderKind::None;
        plan.loader_version = None;

        assert_eq!(version_id(&plan), "1.21.6");
        assert_eq!(version_label(&plan), "Minecraft 1.21.6");
    }

    #[test]
    fn ensure_profile_preserves_unknown_fields_and_is_idempotent() {
        let _guard = ENV_LOCK.lock().expect("test env lock poisoned");
        let home = test_home("ensure-profile");
        let previous_home = env::var_os("HOME");
        set_home(&home);
        let minecraft_dir = default_minecraft_dir_in(&home);
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
        let game_dir =
            home.join("Library/Application Support/Minecraft Setup Manager/Example Server");
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
            "Example Server"
        );

        let second_result = adapter
            .ensure_profile(&plan, &game_dir)
            .expect("ensure profile again");
        assert!(matches!(
            second_result.action,
            LauncherProfileAction::Unchanged
        ));
        assert!(second_result.backup_path.is_none());

        restore_home(previous_home);
    }

    #[test]
    fn ensure_profile_fails_before_write_when_fabric_version_is_missing() {
        let _guard = ENV_LOCK.lock().expect("test env lock poisoned");
        let home = test_home("missing-fabric");
        let previous_home = env::var_os("HOME");
        set_home(&home);
        let minecraft_dir = default_minecraft_dir_in(&home);
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

        restore_home(previous_home);
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
            launcher: LauncherKind::Official,
            profile: "balanced".to_string(),
            profile_label: "Balanced".to_string(),
            recommended_memory_mb: 4096,
            update_status: ServerUpdateStatus::NewSetup,
            actions: vec![],
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

    fn default_minecraft_dir_in(home: &Path) -> PathBuf {
        home.join("Library")
            .join("Application Support")
            .join("minecraft")
    }

    fn read_json(path: &Path) -> serde_json::Value {
        serde_json::from_str(&std::fs::read_to_string(path).expect("read json"))
            .expect("parse json")
    }

    fn set_home(home: &Path) {
        unsafe {
            env::set_var("HOME", home);
        }
    }

    fn restore_home(previous_home: Option<std::ffi::OsString>) {
        unsafe {
            if let Some(value) = previous_home {
                env::set_var("HOME", value);
            } else {
                env::remove_var("HOME");
            }
        }
    }
}
