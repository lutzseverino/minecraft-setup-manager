mod manual;
mod official_minecraft;
mod sklauncher;

use std::path::{Path, PathBuf};

pub use manual::ManualLauncherAdapter;
pub use official_minecraft::OfficialMinecraftLauncherAdapter;
pub use sklauncher::SklauncherAdapter;

use crate::commands::{InstallPlan, LauncherDetection, LauncherKind};
use crate::minecraft::version;
use crate::system::paths;

pub trait LauncherAdapter {
    fn detection(&self) -> LauncherDetection;
}

#[derive(Debug, Clone)]
pub struct LauncherProfileResult {
    pub profile_id: String,
    pub launcher_profiles_path: Option<PathBuf>,
    pub backup_path: Option<PathBuf>,
    pub game_dir: PathBuf,
    pub version_id: String,
    pub action: LauncherProfileAction,
    pub log: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum LauncherProfileAction {
    Created,
    Updated,
    Unchanged,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct LauncherProfileValidation {
    pub required: bool,
    pub launcher_profiles_path: Option<PathBuf>,
    pub launcher_profiles_exists: bool,
    pub version_exists: bool,
    pub profile_exists: bool,
    pub game_dir_matches: bool,
    pub version_matches: bool,
    pub profile_id: String,
    pub expected_game_dir: PathBuf,
    pub expected_version_id: String,
}

pub fn detect_launchers() -> Vec<LauncherDetection> {
    let adapters: Vec<Box<dyn LauncherAdapter>> = vec![
        Box::new(OfficialMinecraftLauncherAdapter),
        Box::new(SklauncherAdapter),
        Box::new(ManualLauncherAdapter),
    ];

    adapters
        .into_iter()
        .map(|adapter| adapter.detection())
        .collect()
}

pub fn validate_profile_prerequisites(plan: &InstallPlan) -> Result<(), String> {
    match plan.launcher {
        LauncherKind::Official => official_minecraft::validate_profile_prerequisites(plan),
        LauncherKind::Sklauncher | LauncherKind::Manual => Ok(()),
    }
}

pub fn validate_base_prerequisites(plan: &InstallPlan) -> Result<(), String> {
    match plan.launcher {
        LauncherKind::Official => official_minecraft::validate_base_prerequisites(plan),
        LauncherKind::Sklauncher | LauncherKind::Manual => Ok(()),
    }
}

pub fn ensure_profile(
    plan: &InstallPlan,
    game_dir: &Path,
) -> Result<LauncherProfileResult, String> {
    match plan.launcher {
        LauncherKind::Official => OfficialMinecraftLauncherAdapter.ensure_profile(plan, game_dir),
        LauncherKind::Sklauncher | LauncherKind::Manual => {
            let version_id = version::installed_version_id(plan);

            Ok(LauncherProfileResult {
                profile_id: plan.server_id.clone(),
                launcher_profiles_path: None,
                backup_path: None,
                game_dir: game_dir.to_path_buf(),
                version_id,
                action: LauncherProfileAction::Skipped,
                log: vec!["Launcher profile changes were skipped for manual setup.".to_string()],
            })
        }
    }
}

pub fn validate_profile(
    plan: &InstallPlan,
    game_dir: &Path,
) -> Result<LauncherProfileValidation, String> {
    match plan.launcher {
        LauncherKind::Official => OfficialMinecraftLauncherAdapter.validate_profile(plan, game_dir),
        LauncherKind::Sklauncher | LauncherKind::Manual => {
            let version_id = version::installed_version_id(plan);
            let version_exists = paths::minecraft_version_file(&version_id)
                .map(|path| path.is_file())
                .unwrap_or(false);

            Ok(LauncherProfileValidation {
                required: false,
                launcher_profiles_path: None,
                launcher_profiles_exists: false,
                version_exists,
                profile_exists: false,
                game_dir_matches: false,
                version_matches: false,
                profile_id: plan.server_id.clone(),
                expected_game_dir: game_dir.to_path_buf(),
                expected_version_id: version_id,
            })
        }
    }
}
