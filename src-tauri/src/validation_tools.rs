use std::env;
use std::fs;
use std::path::Path;

use serde_json::{json, Map, Value};

use crate::commands::{InstallPlan, LauncherKind, ServerUpdateStatus};
use crate::launcher::{LauncherAdapter, LauncherProfileAction, SklauncherAdapter};
use crate::manifest::schema::ManifestLoaderKind;
use crate::system::paths;

const VALIDATION_GUARD: &str = "MSM_SKLAUNCHER_VALIDATION";
const SANDBOX_MARKER: &str = ".minecraft-setup-manager-sklauncher-sandbox";
const ROOT_SENTINEL: &str = "minecraftSetupManagerValidation";
const PROFILE_SENTINEL: &str = "minecraftSetupManagerValidation";
const MINECRAFT_VERSION: &str = "1.21.6";

pub fn run_sklauncher_contract_probe() -> Result<(), String> {
    require_validation_sandbox()?;
    let phase = env::args()
        .nth(1)
        .ok_or_else(|| "Expected a phase: write or verify.".to_string())?;
    let plan = validation_plan();
    let minecraft_dir = paths::default_minecraft_dir()?;
    let game_dir = minecraft_dir.join("minecraft-setup-manager-validation");

    match phase.as_str() {
        "write" => write_phase(&plan, &minecraft_dir, &game_dir),
        "verify" => verify_phase(&plan, &minecraft_dir, &game_dir),
        _ => Err(format!(
            "Unsupported phase {phase:?}; expected write or verify."
        )),
    }
}

fn write_phase(plan: &InstallPlan, minecraft_dir: &Path, game_dir: &Path) -> Result<(), String> {
    let detection = SklauncherAdapter.detection();
    if !detection.setup_supported {
        return Err(format!(
            "The clean launcher workspace was not detected: {}",
            detection.detail
        ));
    }

    let profiles_path = paths::minecraft_launcher_profiles_file()?;
    let mut root = read_json(&profiles_path)?;
    object_mut(&mut root)?.insert(
        ROOT_SENTINEL.to_string(),
        json!({"contractVersion": 1, "preserve": true}),
    );
    write_json(&profiles_path, &root)?;

    prepare_version_fixture(minecraft_dir)?;
    fs::create_dir_all(game_dir).map_err(|error| {
        format!(
            "Could not create validation game directory at {}: {error}",
            game_dir.display()
        )
    })?;

    let result = SklauncherAdapter.ensure_profile(plan, game_dir)?;
    if !matches!(result.action, LauncherProfileAction::Created) {
        return Err(format!(
            "Expected a clean profile creation, got {:?}.",
            result.action
        ));
    }

    let backup_path = result
        .backup_path
        .as_ref()
        .ok_or_else(|| "Profile creation did not produce a backup.".to_string())?;
    if !backup_path.is_file() {
        return Err(format!(
            "Profile backup does not exist at {}.",
            backup_path.display()
        ));
    }

    let mut root = read_json(&profiles_path)?;
    let profile = profile_mut(&mut root, &result.profile_id)?;
    profile.insert(
        PROFILE_SENTINEL.to_string(),
        Value::String("preserve".to_string()),
    );
    write_json(&profiles_path, &root)?;

    print_evidence(json!({
        "phase": "write",
        "launcherProfilesPath": profiles_path,
        "backupPath": backup_path,
        "gameDirectory": result.game_dir,
        "profileId": result.profile_id,
        "versionId": result.version_id,
    }))
}

fn verify_phase(plan: &InstallPlan, _minecraft_dir: &Path, game_dir: &Path) -> Result<(), String> {
    let profiles_path = paths::minecraft_launcher_profiles_file()?;
    let root = read_json(&profiles_path)?;
    require_root_sentinel(&root)?;
    let profile_id = validation_profile_id();
    let launcher_preserved_unknown_profile_field = root
        .get("profiles")
        .and_then(Value::as_object)
        .and_then(|profiles| profiles.get(&profile_id))
        .and_then(Value::as_object)
        .and_then(|profile| profile.get(PROFILE_SENTINEL))
        .and_then(Value::as_str)
        == Some("preserve");

    let result = SklauncherAdapter.ensure_profile(plan, game_dir)?;
    if !matches!(result.action, LauncherProfileAction::Unchanged) {
        return Err(format!(
            "Expected the reloaded profile to be unchanged, got {:?}.",
            result.action
        ));
    }
    if result.backup_path.is_some() {
        return Err("An unchanged profile unexpectedly produced another backup.".to_string());
    }

    let validation = SklauncherAdapter.validate_profile(plan, game_dir)?;
    let passed = validation.required
        && validation.launcher_profiles_exists
        && validation.version_exists
        && validation.profile_exists
        && validation.game_dir_matches
        && validation.version_matches;
    if !passed {
        return Err(format!(
            "The SKlauncher profile did not pass round-trip validation: {validation:?}"
        ));
    }

    print_evidence(json!({
        "phase": "verify",
        "launcherProfilesPath": profiles_path,
        "gameDirectory": validation.expected_game_dir,
        "profileId": validation.profile_id,
        "versionId": validation.expected_version_id,
        "rootSentinelPreserved": true,
        "launcherPreservedUnknownProfileField": launcher_preserved_unknown_profile_field,
        "profileUnchangedAfterReload": true,
    }))
}

fn require_validation_sandbox() -> Result<(), String> {
    if env::var(VALIDATION_GUARD).as_deref() != Ok("1") {
        return Err(format!("Refusing to run without {VALIDATION_GUARD}=1."));
    }

    let home = paths::home_dir()?;
    let marker = home.join(SANDBOX_MARKER);
    if !marker.is_file() {
        return Err(format!(
            "Refusing to modify launcher data without sandbox marker {}.",
            marker.display()
        ));
    }

    Ok(())
}

fn prepare_version_fixture(minecraft_dir: &Path) -> Result<(), String> {
    let version_path = paths::minecraft_version_file(MINECRAFT_VERSION)?;
    let parent = version_path
        .parent()
        .ok_or_else(|| "Validation version path has no parent directory.".to_string())?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "Could not create validation version directory at {}: {error}",
            parent.display()
        )
    })?;
    write_json(
        &version_path,
        &json!({
            "id": MINECRAFT_VERSION,
            "type": "release",
            "mainClass": "net.minecraft.client.main.Main",
            "minecraftSetupManagerValidation": true,
        }),
    )?;

    if !version_path.starts_with(minecraft_dir) {
        return Err("Validation version fixture escaped the Minecraft directory.".to_string());
    }

    Ok(())
}

fn require_root_sentinel(root: &Value) -> Result<(), String> {
    if root
        .get(ROOT_SENTINEL)
        .and_then(|value| value.get("preserve"))
        .and_then(Value::as_bool)
        != Some(true)
    {
        return Err("SKlauncher did not preserve the root validation sentinel.".to_string());
    }

    Ok(())
}

fn validation_profile_id() -> String {
    use sha2::{Digest, Sha256};

    Sha256::digest(validation_plan().server_id.as_bytes())[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn profile_mut<'a>(
    root: &'a mut Value,
    profile_id: &str,
) -> Result<&'a mut Map<String, Value>, String> {
    object_mut(root)?
        .get_mut("profiles")
        .and_then(Value::as_object_mut)
        .and_then(|profiles| profiles.get_mut(profile_id))
        .and_then(Value::as_object_mut)
        .ok_or_else(|| format!("Created profile {profile_id} is missing or invalid."))
}

fn object_mut(value: &mut Value) -> Result<&mut Map<String, Value>, String> {
    value
        .as_object_mut()
        .ok_or_else(|| "Launcher profiles root is not a JSON object.".to_string())
}

fn read_json(path: &Path) -> Result<Value, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("Could not parse {}: {error}", path.display()))
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let contents = serde_json::to_string_pretty(value)
        .map_err(|error| format!("Could not serialize {}: {error}", path.display()))?;
    fs::write(path, format!("{contents}\n"))
        .map_err(|error| format!("Could not write {}: {error}", path.display()))
}

fn print_evidence(value: Value) -> Result<(), String> {
    let output = serde_json::to_string_pretty(&value)
        .map_err(|error| format!("Could not serialize validation evidence: {error}"))?;
    println!("{output}");
    Ok(())
}

fn validation_plan() -> InstallPlan {
    InstallPlan {
        server_id: "sklauncher-contract-validation".to_string(),
        minecraft_version: MINECRAFT_VERSION.to_string(),
        loader_kind: ManifestLoaderKind::None,
        loader_version: None,
        game_directory_name: "SKlauncher Contract Validation".to_string(),
        server_name: "SKlauncher Contract Validation".to_string(),
        server_address: "validation.invalid".to_string(),
        launcher_profile_name: "Minecraft Setup Manager Validation".to_string(),
        launcher: LauncherKind::Sklauncher,
        profile: "validation".to_string(),
        profile_label: "Validation".to_string(),
        recommended_memory_mb: 2048,
        update_status: ServerUpdateStatus::NewSetup,
        actions: vec![],
        resources: vec![],
        required_mods: vec![],
        optional_mods: vec![],
        warnings: vec![],
    }
}
