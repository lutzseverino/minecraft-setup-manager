use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use fastnbt::Value;

use crate::manifest::schema::ManifestServerEntry;
use crate::system::{atomic_file, path_safety};

const SERVERS_FILE_NAME: &str = "servers.dat";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerEntryAction {
    NotRequested,
    Created,
    Updated,
    Unchanged,
}

#[derive(Debug, Clone)]
pub struct ServerEntryResult {
    pub action: ServerEntryAction,
    pub path: PathBuf,
    pub backup_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ServerEntryValidation {
    pub required: bool,
    pub path: PathBuf,
    pub file_exists: bool,
    pub entry_matches: bool,
}

pub fn ensure_server_entry(
    game_dir: &Path,
    entry: Option<&ManifestServerEntry>,
) -> Result<ServerEntryResult, String> {
    let path = game_dir.join(SERVERS_FILE_NAME);
    let Some(entry) = entry else {
        return Ok(ServerEntryResult {
            action: ServerEntryAction::NotRequested,
            path,
            backup_path: None,
        });
    };
    path_safety::reject_symlink(&path, "Minecraft server list")?;

    let existed = path.is_file();
    let mut root = read_root(&path)?;
    let servers = servers_list_mut(&mut root)?;
    let existing_index = servers
        .iter()
        .position(|value| server_address(value) == Some(entry.address.as_str()));
    let action = match existing_index {
        Some(index) => {
            let current = servers[index].clone();
            let compound = server_compound_mut(&mut servers[index])?;
            compound.insert("name".to_string(), Value::String(entry.name.clone()));
            compound.insert("ip".to_string(), Value::String(entry.address.clone()));
            if servers[index] == current {
                ServerEntryAction::Unchanged
            } else {
                ServerEntryAction::Updated
            }
        }
        None => {
            servers.push(Value::Compound(HashMap::from([
                ("name".to_string(), Value::String(entry.name.clone())),
                ("ip".to_string(), Value::String(entry.address.clone())),
            ])));
            ServerEntryAction::Created
        }
    };

    if matches!(action, ServerEntryAction::Unchanged) {
        return Ok(ServerEntryResult {
            action,
            path,
            backup_path: None,
        });
    }

    let backup_path = if existed {
        Some(backup_servers_file(&path)?)
    } else {
        None
    };
    let contents = fastnbt::to_bytes(&root)
        .map_err(|error| format!("Could not prepare the Minecraft server list: {error}"))?;
    atomic_file::write(&path, &contents, "Minecraft server list")?;

    Ok(ServerEntryResult {
        action,
        path,
        backup_path,
    })
}

pub fn validate_server_entry(
    game_dir: &Path,
    entry: Option<&ManifestServerEntry>,
) -> Result<ServerEntryValidation, String> {
    let path = game_dir.join(SERVERS_FILE_NAME);
    let Some(entry) = entry else {
        return Ok(ServerEntryValidation {
            required: false,
            file_exists: path.is_file(),
            entry_matches: false,
            path,
        });
    };
    let file_exists = path.is_file();
    let entry_matches = if file_exists {
        let root = read_root(&path)?;
        root.get("servers")
            .and_then(|value| match value {
                Value::List(servers) => Some(servers),
                _ => None,
            })
            .is_some_and(|servers| {
                servers.iter().any(|value| {
                    server_address(value) == Some(entry.address.as_str())
                        && server_name(value) == Some(entry.name.as_str())
                })
            })
    } else {
        false
    };

    Ok(ServerEntryValidation {
        required: true,
        path,
        file_exists,
        entry_matches,
    })
}

fn read_root(path: &Path) -> Result<HashMap<String, Value>, String> {
    if !path.is_file() {
        return Ok(HashMap::new());
    }
    let contents = fs::read(path).map_err(|error| {
        format!(
            "Could not read the Minecraft server list at {}: {error}",
            path.display()
        )
    })?;
    fastnbt::from_bytes(&contents).map_err(|error| {
        format!(
            "Could not read the Minecraft server list format at {}: {error}",
            path.display()
        )
    })
}

fn servers_list_mut(root: &mut HashMap<String, Value>) -> Result<&mut Vec<Value>, String> {
    let servers = root
        .entry("servers".to_string())
        .or_insert_with(|| Value::List(Vec::new()));
    match servers {
        Value::List(values) => Ok(values),
        _ => Err("The Minecraft server list has an unexpected servers value.".to_string()),
    }
}

fn server_compound_mut(value: &mut Value) -> Result<&mut HashMap<String, Value>, String> {
    match value {
        Value::Compound(compound) => Ok(compound),
        _ => Err("The Minecraft server list contains an invalid server entry.".to_string()),
    }
}

fn server_address(value: &Value) -> Option<&str> {
    let Value::Compound(compound) = value else {
        return None;
    };
    match compound.get("ip") {
        Some(Value::String(address)) => Some(address),
        _ => None,
    }
}

fn server_name(value: &Value) -> Option<&str> {
    let Value::Compound(compound) = value else {
        return None;
    };
    match compound.get("name") {
        Some(Value::String(name)) => Some(name),
        _ => None,
    }
}

fn backup_servers_file(path: &Path) -> Result<PathBuf, String> {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    let backup_path =
        path.with_file_name(format!("servers.minecraft-setup-manager-{unique}.dat.bak"));
    fs::copy(path, &backup_path).map_err(|error| {
        format!(
            "Could not back up the Minecraft server list to {}: {error}",
            backup_path.display()
        )
    })?;
    Ok(backup_path)
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn creates_and_reuses_a_server_entry_without_losing_other_data() {
        let game_dir = test_dir("servers-dat");
        let path = game_dir.join(SERVERS_FILE_NAME);
        let original = HashMap::from([
            ("unknownRoot".to_string(), Value::Int(7)),
            (
                "servers".to_string(),
                Value::List(vec![Value::Compound(HashMap::from([
                    ("name".to_string(), Value::String("Other".to_string())),
                    (
                        "ip".to_string(),
                        Value::String("other.example.com".to_string()),
                    ),
                    ("icon".to_string(), Value::String("kept".to_string())),
                ]))]),
            ),
        ]);
        fs::write(
            &path,
            fastnbt::to_bytes(&original).expect("serialize original"),
        )
        .expect("write original");
        let entry = ManifestServerEntry {
            name: "Example".to_string(),
            address: "play.example.com".to_string(),
        };

        let created = ensure_server_entry(&game_dir, Some(&entry)).expect("create entry");
        let unchanged = ensure_server_entry(&game_dir, Some(&entry)).expect("reuse entry");
        let root = read_root(&path).expect("read result");

        assert_eq!(created.action, ServerEntryAction::Created);
        assert!(created.backup_path.is_some_and(|path| path.is_file()));
        assert_eq!(unchanged.action, ServerEntryAction::Unchanged);
        assert_eq!(root.get("unknownRoot"), Some(&Value::Int(7)));
        let servers = match root.get("servers") {
            Some(Value::List(servers)) => servers,
            _ => panic!("missing servers list"),
        };
        assert_eq!(servers.len(), 2);
        assert!(servers
            .iter()
            .any(|value| server_address(value) == Some("other.example.com")));
        assert!(servers
            .iter()
            .any(|value| server_address(value) == Some("play.example.com")));
    }

    #[test]
    fn updates_a_matching_address_and_preserves_unknown_entry_fields() {
        let game_dir = test_dir("update-server-entry");
        let path = game_dir.join(SERVERS_FILE_NAME);
        let original = HashMap::from([(
            "servers".to_string(),
            Value::List(vec![Value::Compound(HashMap::from([
                ("name".to_string(), Value::String("Old".to_string())),
                (
                    "ip".to_string(),
                    Value::String("play.example.com".to_string()),
                ),
                ("icon".to_string(), Value::String("kept".to_string())),
            ]))]),
        )]);
        fs::write(
            &path,
            fastnbt::to_bytes(&original).expect("serialize original"),
        )
        .expect("write original");
        let entry = ManifestServerEntry {
            name: "New".to_string(),
            address: "play.example.com".to_string(),
        };

        let result = ensure_server_entry(&game_dir, Some(&entry)).expect("update entry");
        let root = read_root(&path).expect("read result");
        let server = match &root["servers"] {
            Value::List(servers) => match &servers[0] {
                Value::Compound(server) => server,
                _ => panic!("invalid server"),
            },
            _ => panic!("invalid servers"),
        };

        assert_eq!(result.action, ServerEntryAction::Updated);
        assert_eq!(server.get("name"), Some(&Value::String("New".to_string())));
        assert_eq!(server.get("icon"), Some(&Value::String("kept".to_string())));
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
