use crate::commands::{LauncherDetection, LauncherDetectionStatus, LauncherKind};

use super::LauncherAdapter;

pub struct ManualLauncherAdapter;

impl LauncherAdapter for ManualLauncherAdapter {
    fn detection(&self) -> LauncherDetection {
        LauncherDetection {
            kind: LauncherKind::Manual,
            status: LauncherDetectionStatus::Manual,
            setup_supported: false,
            detail: "Use this if your launcher is not listed.".to_string(),
            confidence: 1.0,
        }
    }
}
