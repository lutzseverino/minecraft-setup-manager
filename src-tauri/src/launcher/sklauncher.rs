use crate::commands::{LauncherDetection, LauncherDetectionStatus, LauncherKind};

use super::LauncherAdapter;

pub struct SklauncherAdapter;

impl LauncherAdapter for SklauncherAdapter {
    fn detection(&self) -> LauncherDetection {
        LauncherDetection {
            kind: LauncherKind::Sklauncher,
            status: LauncherDetectionStatus::NotFound,
            detail: "Not found on this computer.".to_string(),
            confidence: 0.22,
        }
    }
}
