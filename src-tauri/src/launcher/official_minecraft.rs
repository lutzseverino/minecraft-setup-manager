use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::commands::{InstallPlan, LauncherDetection, LauncherDetectionStatus, LauncherKind};
use crate::system::paths;

use super::{
    LauncherAdapter, LauncherProfileAction, LauncherProfileResult, LauncherProfileValidation,
};

pub struct OfficialMinecraftLauncherAdapter;

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
        validate_profile_prerequisites(plan)?;

        let launcher_profiles_path = paths::minecraft_launcher_profiles_file()?;
        let mut root = read_launcher_profiles(&launcher_profiles_path)?;
        let profiles = profiles_object_mut(&mut root)?;
        let profile_id = profile_id(plan);
        let version_id = fabric_version_id(plan);
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
        let version_id = fabric_version_id(plan);
        let fabric_version_exists = paths::minecraft_version_file(&version_id)?.is_file();
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
            fabric_version_exists,
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

    let version_id = fabric_version_id(plan);
    let version_path = paths::minecraft_version_file(&version_id)?;
    if !version_path.is_file() {
        return Err(format!(
            "Fabric {} for Minecraft {} is not installed yet. Install Fabric first, then run setup again.",
            plan.fabric_loader_version, plan.minecraft_version
        ));
    }

    Ok(())
}

pub fn fabric_version_id(plan: &InstallPlan) -> String {
    format!(
        "fabric-loader-{}-{}",
        plan.fabric_loader_version, plan.minecraft_version
    )
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
    if !root.is_object() {
        *root = Value::Object(Map::new());
    }

    let object = root
        .as_object_mut()
        .ok_or_else(|| "Minecraft Launcher profiles file is not a JSON object.".to_string())?;
    let profiles = object
        .entry("profiles")
        .or_insert_with(|| Value::Object(Map::new()));

    if !profiles.is_object() {
        *profiles = Value::Object(Map::new());
    }

    profiles
        .as_object_mut()
        .ok_or_else(|| "Minecraft Launcher profiles entry is not a JSON object.".to_string())
}

fn backup_launcher_profiles(path: &Path) -> Result<PathBuf, String> {
    let backup_path = path.with_file_name(format!(
        "launcher_profiles.maresme-mc-setup-{}.json.bak",
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
    let temp_path = path.with_extension("json.maresme-mc-setup.tmp");

    fs::write(&temp_path, format!("{json}\n")).map_err(|error| {
        format!(
            "Could not write temporary Minecraft Launcher profiles at {}: {error}",
            temp_path.display()
        )
    })?;
    fs::rename(&temp_path, path).map_err(|error| {
        format!(
            "Could not replace Minecraft Launcher profiles at {}: {error}",
            path.display()
        )
    })
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
    current_timestamp()
        .replace([':', '-'], "")
        .replace('.', "")
        .replace('Z', "")
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    use serde_json::json;

    use super::*;
    use crate::commands::{InstallPlan, LauncherKind, PerformanceProfileId};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn ensure_profile_preserves_unknown_fields_and_is_idempotent() {
        let _guard = ENV_LOCK.lock().expect("test env lock poisoned");
        let home = test_home("ensure-profile");
        let previous_home = env::var_os("HOME");
        set_home(&home);
        let minecraft_dir = default_minecraft_dir_in(&home);
        let profiles_path = minecraft_dir.join("launcher_profiles.json");
        let version_id = "fabric-loader-0.19.3-26.1.2";
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
                    "maresme-mc": {
                        "created": "2026-01-01T00:00:00.000Z",
                        "gameDir": "/old/path",
                        "icon": "Crafting_Table",
                        "lastUsed": "2026-01-02T00:00:00.000Z",
                        "lastVersionId": "old-version",
                        "name": "Old Maresme",
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
        let game_dir = home.join("Library/Application Support/Maresme MC Setup/Maresme MC");
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
            written["profiles"]["maresme-mc"]["unknownProfileField"],
            true
        );
        assert_eq!(written["profiles"]["maresme-mc"]["icon"], "Crafting_Table");
        assert_eq!(
            written["profiles"]["maresme-mc"]["gameDir"],
            game_dir.display().to_string()
        );
        assert_eq!(
            written["profiles"]["maresme-mc"]["lastVersionId"],
            version_id
        );
        assert_eq!(written["profiles"]["maresme-mc"]["name"], "Maresme MC");

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

        assert!(error.contains("Fabric 0.19.3 for Minecraft 26.1.2 is not installed"));
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
            server_id: "maresme-mc".to_string(),
            minecraft_version: "26.1.2".to_string(),
            fabric_loader_version: "0.19.3".to_string(),
            game_directory_name: "Maresme MC".to_string(),
            server_name: "Maresme MC".to_string(),
            server_address: "localhost".to_string(),
            launcher: LauncherKind::Official,
            profile: PerformanceProfileId::Balanced,
            steps: vec![],
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
        let path = env::temp_dir().join(format!("maresme-mc-setup-{name}-{unique}"));
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
