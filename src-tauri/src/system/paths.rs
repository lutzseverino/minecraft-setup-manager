use std::env;
use std::path::PathBuf;

pub fn home_dir() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    if let Some(profile) = env::var_os("USERPROFILE") {
        return Ok(PathBuf::from(profile));
    }

    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "Could not find your home folder.".to_string())
}

pub fn app_support_dir(app_name: &str) -> Result<PathBuf, String> {
    let home = home_dir()?;
    Ok(platform_app_support_dir(&home, app_name))
}

#[cfg(target_os = "macos")]
fn platform_app_support_dir(home: &std::path::Path, app_name: &str) -> PathBuf {
    home.join("Library")
        .join("Application Support")
        .join(app_name)
}

#[cfg(target_os = "windows")]
fn platform_app_support_dir(home: &std::path::Path, app_name: &str) -> PathBuf {
    env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join("AppData").join("Roaming"))
        .join(app_name)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn platform_app_support_dir(home: &std::path::Path, app_name: &str) -> PathBuf {
    env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local").join("share"))
        .join(app_name)
}

pub fn default_minecraft_dir() -> Result<PathBuf, String> {
    let home = home_dir()?;
    Ok(platform_minecraft_dir(&home))
}

#[cfg(target_os = "macos")]
fn platform_minecraft_dir(home: &std::path::Path) -> PathBuf {
    home.join("Library")
        .join("Application Support")
        .join("minecraft")
}

#[cfg(target_os = "windows")]
fn platform_minecraft_dir(home: &std::path::Path) -> PathBuf {
    env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join("AppData").join("Roaming"))
        .join(".minecraft")
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn platform_minecraft_dir(home: &std::path::Path) -> PathBuf {
    home.join(".minecraft")
}

pub fn minecraft_launcher_profiles_file() -> Result<PathBuf, String> {
    Ok(default_minecraft_dir()?.join("launcher_profiles.json"))
}

pub fn minecraft_version_file(version_id: &str) -> Result<PathBuf, String> {
    Ok(default_minecraft_dir()?
        .join("versions")
        .join(version_id)
        .join(format!("{version_id}.json")))
}

pub fn desktop_dir() -> Result<PathBuf, String> {
    Ok(home_dir()?.join("Desktop"))
}
