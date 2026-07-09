use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::path_safety;

pub fn write(path: &Path, contents: &[u8], description: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("Could not find the folder for the {description}."))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "Could not create the folder for the {description} at {}: {error}",
            parent.display()
        )
    })?;
    path_safety::reject_symlink(path, description)?;

    let temp_path = temporary_sibling(path, "tmp");
    let result = write_temporary(&temp_path, contents, description)
        .and_then(|()| replace_file(&temp_path, path, description));

    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }

    result
}

pub fn replace_file(temp_path: &Path, target_path: &Path, description: &str) -> Result<(), String> {
    path_safety::reject_symlink(target_path, description)?;
    let initial_error = match fs::rename(temp_path, target_path) {
        Ok(()) => return Ok(()),
        Err(error) => error,
    };

    if !target_path.exists() {
        return Err(format!(
            "Could not install the {description} at {}: {initial_error}",
            target_path.display()
        ));
    }

    let backup_path = temporary_sibling(target_path, "replace-backup");
    fs::rename(target_path, &backup_path).map_err(|error| {
        format!(
            "Could not prepare to replace the {description} at {}: {error}",
            target_path.display()
        )
    })?;

    if let Err(error) = fs::rename(temp_path, target_path) {
        let restore_result = fs::rename(&backup_path, target_path);
        return Err(match restore_result {
            Ok(()) => format!(
                "Could not replace the {description}; the old file was restored: {error}"
            ),
            Err(restore_error) => format!(
                "Could not replace the {description}, and its backup at {} could not be restored: {error}; {restore_error}",
                backup_path.display()
            ),
        });
    }

    fs::remove_file(&backup_path).map_err(|error| {
        format!(
            "The {description} was saved, but its temporary backup at {} could not be removed: {error}",
            backup_path.display()
        )
    })
}

fn write_temporary(path: &Path, contents: &[u8], description: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|error| format!("Could not create a temporary {description}: {error}"))?;
    file.write_all(contents)
        .map_err(|error| format!("Could not write the temporary {description}: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("Could not finish writing the temporary {description}: {error}"))
}

fn temporary_sibling(path: &Path, suffix: &str) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("file");
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    path.with_file_name(format!(
        ".minecraft-setup-manager-{file_name}-{}-{unique}.{suffix}",
        std::process::id()
    ))
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn atomically_creates_and_replaces_a_file() {
        let root = env::temp_dir().join(format!(
            "minecraft-setup-manager-atomic-file-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create test root");
        let path = root.join("state.json");

        write(&path, b"first", "test state").expect("create file");
        write(&path, b"second", "test state").expect("replace file");

        assert_eq!(fs::read(path).expect("read file"), b"second");
    }
}
