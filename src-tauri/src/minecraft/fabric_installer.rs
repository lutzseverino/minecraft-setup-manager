use std::fs;
use std::path::{Path, PathBuf};

use reqwest::Url;
use serde_json::Value;

use crate::commands::InstallPlan;
use crate::http_client;
use crate::manifest::schema::ManifestLoaderKind;
use crate::minecraft::version;
use crate::system::{atomic_file, paths};

const FABRIC_META_BASE_URL: &str = "https://meta.fabricmc.net/";
const MAX_PROFILE_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoaderInstallAction {
    NotNeeded,
    Installed,
    Unchanged,
}

#[derive(Debug, Clone)]
pub struct LoaderInstallResult {
    pub version_id: String,
    pub profile_path: PathBuf,
    pub action: LoaderInstallAction,
}

pub fn ensure_loader(plan: &InstallPlan) -> Result<LoaderInstallResult, String> {
    let minecraft_dir = paths::default_minecraft_dir()?;
    ensure_loader_in(plan, &minecraft_dir, fetch_fabric_profile)
}

fn ensure_loader_in<F>(
    plan: &InstallPlan,
    minecraft_dir: &Path,
    fetch_profile: F,
) -> Result<LoaderInstallResult, String>
where
    F: FnOnce(&InstallPlan) -> Result<Vec<u8>, String>,
{
    let version_id = version::installed_version_id(plan);
    let profile_path = minecraft_dir
        .join("versions")
        .join(&version_id)
        .join(format!("{version_id}.json"));

    if matches!(plan.loader_kind, ManifestLoaderKind::None) {
        return Ok(LoaderInstallResult {
            version_id,
            profile_path,
            action: LoaderInstallAction::NotNeeded,
        });
    }

    let profile_bytes = fetch_profile(plan)?;
    let profile: Value = serde_json::from_slice(&profile_bytes)
        .map_err(|error| format!("Fabric returned an invalid launcher profile: {error}"))?;
    validate_fabric_profile(plan, &profile)?;
    let normalized = serde_json::to_vec_pretty(&profile)
        .map_err(|error| format!("Could not prepare the Fabric launcher profile: {error}"))?;

    if fs::read(&profile_path).is_ok_and(|current| current == normalized) {
        return Ok(LoaderInstallResult {
            version_id,
            profile_path,
            action: LoaderInstallAction::Unchanged,
        });
    }

    atomic_file::write(&profile_path, &normalized, "Fabric launcher profile")?;

    Ok(LoaderInstallResult {
        version_id,
        profile_path,
        action: LoaderInstallAction::Installed,
    })
}

fn fetch_fabric_profile(plan: &InstallPlan) -> Result<Vec<u8>, String> {
    let url = fabric_profile_url(plan)?;
    http_client::get_bytes(url.as_str(), MAX_PROFILE_BYTES, "Fabric launcher profile")
}

fn fabric_profile_url(plan: &InstallPlan) -> Result<Url, String> {
    let loader_version = plan
        .loader_version
        .as_deref()
        .ok_or_else(|| "The Fabric setup is missing its loader version.".to_string())?;
    let mut url = Url::parse(FABRIC_META_BASE_URL)
        .map_err(|error| format!("Could not prepare the Fabric Meta URL: {error}"))?;
    url.path_segments_mut()
        .map_err(|_| "Could not prepare the Fabric Meta URL.".to_string())?
        .extend([
            "v2",
            "versions",
            "loader",
            plan.minecraft_version.as_str(),
            loader_version,
            "profile",
            "json",
        ]);
    Ok(url)
}

fn validate_fabric_profile(plan: &InstallPlan, profile: &Value) -> Result<(), String> {
    let expected_id = version::installed_version_id(plan);
    let id = profile.get("id").and_then(Value::as_str);
    let inherits_from = profile.get("inheritsFrom").and_then(Value::as_str);
    let main_class = profile.get("mainClass").and_then(Value::as_str);
    let libraries = profile.get("libraries").and_then(Value::as_array);

    if id != Some(expected_id.as_str()) {
        return Err("Fabric returned a launcher profile with the wrong version ID.".to_string());
    }
    if inherits_from != Some(plan.minecraft_version.as_str()) {
        return Err(
            "Fabric returned a launcher profile for the wrong Minecraft version.".to_string(),
        );
    }
    if !main_class.is_some_and(|value| value.starts_with("net.fabricmc.loader.")) {
        return Err(
            "Fabric returned a launcher profile with an unexpected main class.".to_string(),
        );
    }
    if libraries.is_none() {
        return Err("Fabric returned a launcher profile without its libraries.".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::commands::{LauncherKind, ServerUpdateStatus};

    #[test]
    fn builds_the_official_encoded_fabric_profile_url() {
        let mut plan = fabric_plan();
        plan.minecraft_version = "1.20 Pre-Release 1".to_string();

        assert_eq!(
            fabric_profile_url(&plan).expect("profile URL").as_str(),
            "https://meta.fabricmc.net/v2/versions/loader/1.20%20Pre-Release%201/0.16.14/profile/json"
        );
    }

    #[test]
    fn installs_and_reuses_a_valid_fabric_profile() {
        let root = test_dir("fabric-profile");
        let plan = fabric_plan();
        let bytes = profile_bytes(&plan);
        let first = ensure_loader_in(&plan, &root, |_| Ok(bytes.clone())).expect("install profile");
        let second = ensure_loader_in(&plan, &root, |_| Ok(bytes)).expect("reuse profile");

        assert_eq!(first.action, LoaderInstallAction::Installed);
        assert_eq!(second.action, LoaderInstallAction::Unchanged);
        assert!(first.profile_path.is_file());
    }

    #[test]
    fn rejects_a_profile_for_another_minecraft_version() {
        let plan = fabric_plan();
        let mut profile: Value =
            serde_json::from_slice(&profile_bytes(&plan)).expect("parse profile");
        profile["inheritsFrom"] = Value::String("1.20.1".to_string());

        let error = validate_fabric_profile(&plan, &profile).expect_err("wrong version must fail");

        assert!(error.contains("wrong Minecraft version"));
    }

    #[test]
    fn vanilla_loader_is_a_no_op() {
        let root = test_dir("vanilla-loader");
        let mut plan = fabric_plan();
        plan.loader_kind = ManifestLoaderKind::None;
        plan.loader_version = None;

        let result = ensure_loader_in(&plan, &root, |_| panic!("must not fetch"))
            .expect("vanilla loader result");

        assert_eq!(result.action, LoaderInstallAction::NotNeeded);
        assert_eq!(result.version_id, plan.minecraft_version);
    }

    fn profile_bytes(plan: &InstallPlan) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "id": version::installed_version_id(plan),
            "inheritsFrom": plan.minecraft_version,
            "mainClass": "net.fabricmc.loader.impl.launch.knot.KnotClient",
            "libraries": [{ "name": "net.fabricmc:fabric-loader:0.16.14" }]
        }))
        .expect("serialize profile")
    }

    fn fabric_plan() -> InstallPlan {
        InstallPlan {
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
            actions: vec![],
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
