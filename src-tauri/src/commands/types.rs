use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LauncherKind {
    Official,
    Sklauncher,
    Manual,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PerformanceProfileId {
    LowEnd,
    Balanced,
    Shaders,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LauncherDetectionStatus {
    Detected,
    NotFound,
    Manual,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallPhase {
    Idle,
    Planning,
    Preparing,
    Installing,
    Validating,
    Complete,
    Failed,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    Pass,
    Warning,
    Fail,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPlanRequest {
    pub server_id: String,
    pub launcher: LauncherKind,
    pub profile: PerformanceProfileId,
    pub server_address: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherDetection {
    pub kind: LauncherKind,
    pub status: LauncherDetectionStatus,
    pub detail: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPlan {
    pub server_id: String,
    pub minecraft_version: String,
    pub fabric_loader_version: String,
    pub game_directory_name: String,
    pub server_name: String,
    pub server_address: String,
    pub launcher: LauncherKind,
    pub profile: PerformanceProfileId,
    pub steps: Vec<String>,
    pub required_mods: Vec<String>,
    pub optional_mods: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgress {
    pub phase: InstallPhase,
    pub percent: u8,
    pub log: Vec<String>,
    pub plan: InstallPlan,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationCheck {
    pub id: String,
    pub label: String,
    pub detail: String,
    pub status: ValidationStatus,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub overall: ValidationStatus,
    pub checks: Vec<ValidationCheck>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticBundle {
    pub path: String,
    pub summary: String,
}
