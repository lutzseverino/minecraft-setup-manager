use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde_json::{Map, Value};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::commands::InstallPlan;
use crate::minecraft::version;
use crate::system::paths;
use crate::system::{atomic_file, path_safety};

use super::{LauncherProfileAction, LauncherProfileResult, LauncherProfileValidation};

static PROFILE_WRITE_LOCK: Mutex<()> = Mutex::new(());

pub fn ensure_profile(
    plan: &InstallPlan,
    game_dir: &Path,
    profile_id: &str,
    launcher_name: &str,
) -> Result<LauncherProfileResult, String> {
    let _guard = PROFILE_WRITE_LOCK
        .lock()
        .map_err(|_| "The launcher profile lock is unavailable.".to_string())?;
    validate_profile_prerequisites(plan, launcher_name)?;

    let launcher_profiles_path = paths::minecraft_launcher_profiles_file()?;
    let mut root = read_launcher_profiles(&launcher_profiles_path)?;
    let profiles = profiles_object_mut(&mut root)?;
    let version_id = version::installed_version_id(plan);
    let expected_game_dir = game_dir.display().to_string();
    let existing_profile = profiles.get(profile_id).cloned();
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
    next_profile.insert(
        "name".to_string(),
        Value::String(plan.launcher_profile_name.clone()),
    );
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

    profiles.insert(profile_id.to_string(), next_value);

    if !matches!(action, LauncherProfileAction::Unchanged) {
        write_launcher_profiles(&launcher_profiles_path, &root)?;
    }

    Ok(LauncherProfileResult {
        profile_id: profile_id.to_string(),
        launcher_profiles_path: Some(launcher_profiles_path),
        backup_path,
        game_dir: game_dir.to_path_buf(),
        version_id,
        action: action.clone(),
        log: vec![profile_log_line(&action, launcher_name)],
    })
}

pub fn validate_profile(
    plan: &InstallPlan,
    game_dir: &Path,
    profile_id: &str,
) -> Result<LauncherProfileValidation, String> {
    let launcher_profiles_path = paths::minecraft_launcher_profiles_file()?;
    let version_id = version::installed_version_id(plan);
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
        .and_then(|profiles| profiles.get(profile_id));
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
        profile_id: profile_id.to_string(),
        expected_game_dir: game_dir.to_path_buf(),
        expected_version_id: version_id,
    })
}

pub fn validate_profile_prerequisites(
    plan: &InstallPlan,
    launcher_name: &str,
) -> Result<(), String> {
    validate_launcher_profiles_file(launcher_name)?;

    let version_id = version::installed_version_id(plan);
    let version_path = paths::minecraft_version_file(&version_id)?;
    if !version_path.is_file() {
        return Err(format!(
            "{} is not installed yet. Open {launcher_name}, install or run that version once, then try again.",
            version::version_label(plan)
        ));
    }

    Ok(())
}

pub fn validate_base_prerequisites(plan: &InstallPlan, launcher_name: &str) -> Result<(), String> {
    validate_launcher_profiles_file(launcher_name)?;
    let base_path = paths::minecraft_version_file(&plan.minecraft_version)?;
    if !base_path.is_file() {
        return Err(format!(
            "Minecraft {} is not installed yet. Open {launcher_name} and run it once, then try again.",
            plan.minecraft_version
        ));
    }
    Ok(())
}

fn validate_launcher_profiles_file(launcher_name: &str) -> Result<(), String> {
    let launcher_profiles_path = paths::minecraft_launcher_profiles_file()?;
    if !launcher_profiles_path.is_file() {
        return Err(format!(
            "Could not find launcher profiles at {}. Open {launcher_name} once, then try again.",
            launcher_profiles_path.display()
        ));
    }

    Ok(())
}

pub(super) fn read_launcher_profiles(path: &Path) -> Result<Value, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "Could not read launcher profiles at {}: {error}",
            path.display()
        )
    })?;

    serde_json::from_str(&contents).map_err(|error| {
        format!(
            "Could not parse launcher profiles at {}: {error}",
            path.display()
        )
    })
}

pub(super) fn profiles_object_mut(root: &mut Value) -> Result<&mut Map<String, Value>, String> {
    let object = root
        .as_object_mut()
        .ok_or_else(|| "Launcher profiles file is not a JSON object.".to_string())?;
    let profiles = object
        .entry("profiles")
        .or_insert_with(|| Value::Object(Map::new()));

    profiles
        .as_object_mut()
        .ok_or_else(|| "Launcher profiles entry is not a JSON object.".to_string())
}

fn backup_launcher_profiles(path: &Path) -> Result<PathBuf, String> {
    path_safety::reject_symlink(path, "launcher profiles file")?;
    let backup_path = path.with_file_name(format!(
        "launcher_profiles.minecraft-setup-manager-{}.json.bak",
        backup_timestamp()
    ));

    fs::copy(path, &backup_path).map_err(|error| {
        format!(
            "Could not back up launcher profiles to {}: {error}",
            backup_path.display()
        )
    })?;

    Ok(backup_path)
}

fn write_launcher_profiles(path: &Path, root: &Value) -> Result<(), String> {
    let json = serde_json::to_string_pretty(root)
        .map_err(|error| format!("Could not prepare launcher profiles: {error}"))?;
    atomic_file::write(
        path,
        format!("{json}\n").as_bytes(),
        "launcher profiles file",
    )
}

fn profile_log_line(action: &LauncherProfileAction, launcher_name: &str) -> String {
    match action {
        LauncherProfileAction::Created => format!("Created the {launcher_name} profile."),
        LauncherProfileAction::Updated => format!("Updated the {launcher_name} profile."),
        LauncherProfileAction::Unchanged => {
            format!("{launcher_name} profile was already ready.")
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
