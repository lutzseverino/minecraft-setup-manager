use std::fs;
use std::path::{Component, Path, PathBuf};

use unicode_normalization::UnicodeNormalization;

const MAX_PORTABLE_NAME_BYTES: usize = 200;

pub fn validate_portable_component(value: &str, label: &str) -> Result<(), String> {
    if value.is_empty() || value != value.trim() {
        return Err(format!(
            "The {label} must not be empty or have outside spaces."
        ));
    }

    if value != value.nfc().collect::<String>()
        || value.len() > MAX_PORTABLE_NAME_BYTES
        || value.ends_with('.')
        || value.contains(['<', '>', ':', '"', '/', '\\', '|', '?', '*'])
        || value.chars().any(char::is_control)
        || !Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(format!("The {label} must be one safe file or folder name."));
    }

    let stem = value
        .split('.')
        .next()
        .unwrap_or(value)
        .to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || stem
            .strip_prefix("COM")
            .or_else(|| stem.strip_prefix("LPT"))
            .is_some_and(|number| {
                matches!(number, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
            });

    if reserved {
        return Err(format!("The {label} uses a name reserved by Windows."));
    }

    Ok(())
}

pub fn safe_child_path(root: &Path, name: &str, label: &str) -> Result<PathBuf, String> {
    validate_portable_component(name, label)?;
    let path = root.join(name);
    reject_symlink(&path, label)?;
    Ok(path)
}

pub fn reject_symlink(path: &Path, label: &str) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(format!(
            "The {label} cannot be a shortcut or symbolic link."
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "Could not safely inspect the {label} at {}: {error}",
            path.display()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_plain_cross_platform_names() {
        assert!(validate_portable_component("Example Server", "folder name").is_ok());
        assert!(validate_portable_component("fabric-api.jar", "file name").is_ok());
    }

    #[test]
    fn rejects_paths_and_windows_reserved_names() {
        for name in [
            "../outside",
            "folder/file",
            "folder\\file",
            "NUL",
            "COM1.jar",
            "bad:",
            "bad?.jar",
            "bad|name",
        ] {
            assert!(validate_portable_component(name, "name").is_err(), "{name}");
        }
    }
}
