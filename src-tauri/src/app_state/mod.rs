use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::commands::{LauncherKind, SavedServerEntry};
use crate::manifest::schema::{
    ManifestResourceHashes, ManifestResourceSource, ManifestResourceTarget, SetupManifest,
};
use crate::server::address::server_key;
use crate::system::{atomic_file, paths, APP_SUPPORT_NAME};

const STATE_FILE_NAME: &str = "state.json";
const MANIFEST_CACHE_DIR: &str = "manifests";
const CURRENT_INSTALL_LAYOUT_VERSION: u16 = 1;
static STATE_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppState {
    schema_version: u16,
    servers: Vec<SavedServerRecord>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SavedServerRecord {
    id: String,
    address: String,
    manifest_url: String,
    display_name: String,
    created_at: String,
    last_checked_at: String,
    last_installed_at: Option<String>,
    #[serde(default)]
    checked_manifest_fingerprint: Option<String>,
    selected_launcher: LauncherKind,
    selected_profile: String,
    game_dir: Option<PathBuf>,
    installed: Option<InstalledManifestRecord>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstalledManifestRecord {
    manifest_id: String,
    manifest_version: String,
    manifest_fingerprint: String,
    #[serde(default)]
    layout_version: u16,
    #[serde(default)]
    resources: Vec<InstalledResourceRecord>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstalledResourceRecord {
    id: String,
    name: String,
    target: ManifestResourceTarget,
    file_name: Option<String>,
    source: ManifestResourceSource,
    hashes: ManifestResourceHashes,
}

#[derive(Debug, Clone)]
pub struct InstalledServerSnapshot {
    pub resources: Vec<InstalledResourceSnapshot>,
}

#[derive(Debug, Clone)]
pub struct InstalledResourceSnapshot {
    pub id: String,
    pub name: String,
    pub target: ManifestResourceTarget,
    pub file_name: Option<String>,
    pub source: ManifestResourceSource,
    pub hashes: ManifestResourceHashes,
}

pub fn list_saved_servers() -> Result<Vec<SavedServerEntry>, String> {
    let _guard = lock_state()?;
    let state = read_state()?;

    Ok(state.servers.into_iter().map(Into::into).collect())
}

pub fn saved_server_entry(server_id: &str) -> Result<SavedServerEntry, String> {
    let _guard = lock_state()?;
    let state = read_state()?;

    state
        .servers
        .into_iter()
        .find(|server| server.id == server_id)
        .map(Into::into)
        .ok_or_else(|| "Choose or add a server before starting setup.".to_string())
}

pub fn installed_server_snapshot(
    server_id: &str,
) -> Result<Option<InstalledServerSnapshot>, String> {
    let _guard = lock_state()?;
    let state = read_state()?;

    Ok(state
        .servers
        .into_iter()
        .find(|server| server.id == server_id)
        .and_then(|server| server.installed.map(Into::into)))
}

pub fn saved_manifest_snapshot(server_id: &str) -> Result<SetupManifest, String> {
    let _guard = lock_state()?;
    let state = read_state()?;
    let record = state
        .servers
        .into_iter()
        .find(|server| server.id == server_id)
        .ok_or_else(|| "Choose or add a server before starting setup.".to_string())?;
    let expected_fingerprint = record.checked_manifest_fingerprint.ok_or_else(|| {
        "Check this server again before reviewing or applying its setup.".to_string()
    })?;
    let path = manifest_cache_path(&record.id)?;
    let contents = fs::read(&path).map_err(|error| {
        format!(
            "Could not read the saved setup file at {}: {error}. Check the server again.",
            path.display()
        )
    })?;
    let manifest: SetupManifest = serde_json::from_slice(&contents)
        .map_err(|error| format!("The saved setup file is damaged: {error}"))?;
    crate::manifest::validation::validate_manifest(&manifest, &record.manifest_url)?;
    let actual_fingerprint = crate::manifest::fingerprint::manifest_fingerprint(&manifest)?;

    if actual_fingerprint != expected_fingerprint
        || server_key(&record.address, &manifest.id) != record.id
    {
        return Err(
            "The saved setup file no longer matches this server. Check the server again."
                .to_string(),
        );
    }

    Ok(manifest)
}

impl From<InstalledManifestRecord> for InstalledServerSnapshot {
    fn from(record: InstalledManifestRecord) -> Self {
        Self {
            resources: record.resources.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<InstalledResourceRecord> for InstalledResourceSnapshot {
    fn from(record: InstalledResourceRecord) -> Self {
        Self {
            id: record.id,
            name: record.name,
            target: record.target,
            file_name: record.file_name,
            source: record.source,
            hashes: record.hashes,
        }
    }
}

pub fn record_installed_server(
    server_id: &str,
    selected_launcher: LauncherKind,
    selected_profile: &str,
    game_dir: PathBuf,
    manifest: &SetupManifest,
    manifest_fingerprint: &str,
    resources: Vec<InstalledResourceSnapshot>,
) -> Result<SavedServerEntry, String> {
    let _guard = lock_state()?;
    let mut state = read_state()?;
    let now = timestamp();
    let record = state
        .servers
        .iter_mut()
        .find(|server| server.id == server_id)
        .ok_or_else(|| "Choose or add a server before starting setup.".to_string())?;

    record.display_name = manifest.display_name.clone();
    record.last_installed_at = Some(now);
    record.selected_launcher = selected_launcher;
    record.selected_profile = selected_profile.to_string();
    record.game_dir = Some(game_dir);
    record.installed = Some(InstalledManifestRecord {
        manifest_id: manifest.id.clone(),
        manifest_version: manifest.manifest_version.clone(),
        manifest_fingerprint: manifest_fingerprint.to_string(),
        layout_version: CURRENT_INSTALL_LAYOUT_VERSION,
        resources: resources.into_iter().map(Into::into).collect(),
    });
    state.schema_version = 4;
    let entry = record.clone().into();
    write_state(&state)?;

    Ok(entry)
}

pub fn upsert_checked_server(
    address: &str,
    manifest_url: &str,
    manifest: &SetupManifest,
    fingerprint: &str,
) -> Result<SavedServerEntry, String> {
    let _guard = lock_state()?;
    let mut state = read_state()?;
    let now = timestamp();
    let id = server_key(address, &manifest.id);
    let default_profile = default_profile_id(manifest)?.to_string();
    write_manifest_cache(&id, manifest)?;

    if let Some(record) = state.servers.iter_mut().find(|server| server.id == id) {
        record.address = address.to_string();
        record.manifest_url = manifest_url.to_string();
        record.display_name = manifest.display_name.clone();
        record.last_checked_at = now;
        record.checked_manifest_fingerprint = Some(fingerprint.to_string());
        if !manifest
            .profiles
            .iter()
            .any(|profile| profile.id == record.selected_profile)
        {
            record.selected_profile = default_profile;
        }
        let entry = record.clone().into();
        write_state(&state)?;
        return Ok(entry);
    }

    let record = SavedServerRecord {
        id,
        address: address.to_string(),
        manifest_url: manifest_url.to_string(),
        display_name: manifest.display_name.clone(),
        created_at: now.clone(),
        last_checked_at: now,
        last_installed_at: None,
        checked_manifest_fingerprint: Some(fingerprint.to_string()),
        selected_launcher: LauncherKind::Official,
        selected_profile: default_profile,
        game_dir: None,
        installed: None,
    };
    let entry = record.clone().into();
    state.servers.push(record);
    write_state(&state)?;

    Ok(entry)
}

fn read_state() -> Result<AppState, String> {
    let path = state_path()?;

    if !path.is_file() {
        return Ok(AppState {
            schema_version: 1,
            servers: Vec::new(),
        });
    }

    let contents = fs::read_to_string(&path).map_err(|error| {
        format!(
            "Could not read saved servers at {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_str(&contents).map_err(|error| {
        format!(
            "Could not parse saved servers at {}: {error}",
            path.display()
        )
    })
}

fn write_state(state: &AppState) -> Result<(), String> {
    let path = state_path()?;
    let parent = path
        .parent()
        .ok_or_else(|| "Could not find the app data folder.".to_string())?;

    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "Could not create the app data folder at {}: {error}",
            parent.display()
        )
    })?;
    let contents = serde_json::to_vec_pretty(state)
        .map_err(|error| format!("Could not prepare saved servers: {error}"))?;
    atomic_file::write(&path, &contents, "saved server state")
}

fn state_path() -> Result<PathBuf, String> {
    Ok(paths::app_support_dir(APP_SUPPORT_NAME)?.join(STATE_FILE_NAME))
}

fn manifest_cache_path(server_id: &str) -> Result<PathBuf, String> {
    let digest = Sha256::digest(server_id.as_bytes());
    let file_name = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(paths::app_support_dir(APP_SUPPORT_NAME)?
        .join(MANIFEST_CACHE_DIR)
        .join(format!("{file_name}.json")))
}

fn write_manifest_cache(server_id: &str, manifest: &SetupManifest) -> Result<(), String> {
    let path = manifest_cache_path(server_id)?;
    let contents = serde_json::to_vec_pretty(manifest)
        .map_err(|error| format!("Could not prepare the saved setup file: {error}"))?;
    atomic_file::write(&path, &contents, "saved setup file")
}

fn lock_state() -> Result<std::sync::MutexGuard<'static, ()>, String> {
    STATE_LOCK
        .lock()
        .map_err(|_| "The saved server state lock is unavailable.".to_string())
}

fn timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn default_profile_id(manifest: &SetupManifest) -> Result<&str, String> {
    manifest
        .profiles
        .first()
        .map(|profile| profile.id.as_str())
        .ok_or_else(|| "The server setup does not provide any setup options.".to_string())
}

impl From<SavedServerRecord> for SavedServerEntry {
    fn from(record: SavedServerRecord) -> Self {
        let installed = record.installed;
        let needs_repair = installed
            .as_ref()
            .is_some_and(|installed| installed.layout_version != CURRENT_INSTALL_LAYOUT_VERSION);

        Self {
            id: record.id,
            address: record.address,
            manifest_url: record.manifest_url,
            display_name: record.display_name,
            last_checked_at: record.last_checked_at,
            last_installed_at: record.last_installed_at,
            selected_launcher: record.selected_launcher,
            selected_profile: record.selected_profile,
            installed_manifest_version: installed
                .as_ref()
                .map(|installed| installed.manifest_version.clone()),
            installed_manifest_fingerprint: installed
                .as_ref()
                .map(|installed| installed.manifest_fingerprint.clone()),
            needs_repair,
        }
    }
}

impl From<InstalledResourceSnapshot> for InstalledResourceRecord {
    fn from(snapshot: InstalledResourceSnapshot) -> Self {
        Self {
            id: snapshot.id,
            name: snapshot.name,
            target: snapshot.target,
            file_name: snapshot.file_name,
            source: snapshot.source,
            hashes: snapshot.hashes,
        }
    }
}
