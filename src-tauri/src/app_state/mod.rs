use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::commands::{LauncherKind, PerformanceProfileId, SavedServerEntry};
use crate::manifest::schema::{
    ManifestResourceHashes, ManifestResourceSource, ManifestResourceTarget, SetupManifest,
};
use crate::server::address::server_key;
use crate::system::{paths, APP_SUPPORT_NAME};

const STATE_FILE_NAME: &str = "state.json";

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
    selected_launcher: LauncherKind,
    selected_profile: PerformanceProfileId,
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
    resources: Vec<InstalledResourceRecord>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstalledResourceRecord {
    id: String,
    name: String,
    target: ManifestResourceTarget,
    source: ManifestResourceSource,
    hashes: ManifestResourceHashes,
}

pub fn list_saved_servers() -> Result<Vec<SavedServerEntry>, String> {
    let state = read_state()?;

    Ok(state.servers.into_iter().map(Into::into).collect())
}

pub fn saved_server_entry(server_id: &str) -> Result<SavedServerEntry, String> {
    let state = read_state()?;

    state
        .servers
        .into_iter()
        .find(|server| server.id == server_id)
        .map(Into::into)
        .ok_or_else(|| "Choose or add a server before starting setup.".to_string())
}

pub fn saved_server_manifest_url(server_id: &str) -> Result<String, String> {
    let state = read_state()?;

    state
        .servers
        .into_iter()
        .find(|server| server.id == server_id)
        .map(|server| server.manifest_url)
        .ok_or_else(|| "Choose or add a server before starting setup.".to_string())
}

pub fn record_installed_server(
    server_id: &str,
    selected_launcher: LauncherKind,
    selected_profile: PerformanceProfileId,
    game_dir: PathBuf,
    manifest: &SetupManifest,
    manifest_fingerprint: &str,
) -> Result<SavedServerEntry, String> {
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
    record.selected_profile = selected_profile;
    record.game_dir = Some(game_dir);
    record.installed = Some(InstalledManifestRecord {
        manifest_id: manifest.id.clone(),
        manifest_version: manifest.manifest_version.clone(),
        manifest_fingerprint: manifest_fingerprint.to_string(),
        resources: installed_resource_records(manifest, selected_profile),
    });
    state.schema_version = 2;
    let entry = record.clone().into();
    write_state(&state)?;

    Ok(entry)
}

pub fn upsert_checked_server(
    address: &str,
    manifest_url: &str,
    manifest: &SetupManifest,
    _fingerprint: &str,
) -> Result<SavedServerEntry, String> {
    let mut state = read_state()?;
    let now = timestamp();
    let id = server_key(address, &manifest.id);

    if let Some(record) = state.servers.iter_mut().find(|server| server.id == id) {
        record.address = address.to_string();
        record.manifest_url = manifest_url.to_string();
        record.display_name = manifest.display_name.clone();
        record.last_checked_at = now;
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
        selected_launcher: LauncherKind::Official,
        selected_profile: PerformanceProfileId::Balanced,
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
    fs::write(
        &path,
        serde_json::to_string_pretty(state)
            .map_err(|error| format!("Could not prepare saved servers: {error}"))?,
    )
    .map_err(|error| format!("Could not save servers at {}: {error}", path.display()))
}

fn state_path() -> Result<PathBuf, String> {
    Ok(paths::app_support_dir(APP_SUPPORT_NAME)?.join(STATE_FILE_NAME))
}

fn timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn installed_resource_records(
    manifest: &SetupManifest,
    profile: PerformanceProfileId,
) -> Vec<InstalledResourceRecord> {
    crate::manifest::selected_resources(manifest, profile)
        .into_iter()
        .map(|resource| InstalledResourceRecord {
            id: resource.id.clone(),
            name: resource.name.clone(),
            target: resource.target.clone(),
            source: resource.source.clone(),
            hashes: resource.hashes.clone(),
        })
        .collect()
}

impl From<SavedServerRecord> for SavedServerEntry {
    fn from(record: SavedServerRecord) -> Self {
        let installed = record.installed;

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
        }
    }
}
