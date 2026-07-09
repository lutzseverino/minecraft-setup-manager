use std::fs;
use std::path::PathBuf;

use serde::Serialize;

use crate::commands::InstallPlan;
use crate::manifest::schema::SetupManifest;
use crate::minecraft::managed_resources::{self, ManagedResourceAction};
use crate::system::{atomic_file, paths, APP_SUPPORT_NAME};

const RECEIPT_FILE_NAME: &str = "minecraft-setup-manager.json";

#[derive(Debug, Clone)]
pub struct LocalInstallResult {
    pub game_dir: PathBuf,
    pub receipt_path: PathBuf,
    pub resource_results: Vec<managed_resources::ManagedResourceResult>,
    pub log: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LocalValidationResult {
    pub game_dir_exists: bool,
    pub mods_dir_exists: bool,
    pub receipt_exists: bool,
    pub receipt_matches: bool,
    pub game_dir: PathBuf,
    pub receipt_path: PathBuf,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SetupReceipt<'a> {
    manifest_id: &'a str,
    manifest_version: &'a str,
    server_id: &'a str,
    server_name: &'a str,
    server_address: &'a str,
    minecraft_version: &'a str,
    loader_kind: crate::manifest::schema::ManifestLoaderKind,
    loader_version: Option<&'a str>,
    game_directory_name: &'a str,
    launcher: String,
    performance_profile: String,
    required_mods: &'a [String],
    optional_mods: &'a [String],
}

pub fn prepare_local_install(
    plan: &InstallPlan,
    manifest: &SetupManifest,
) -> Result<LocalInstallResult, String> {
    let game_dir = game_directory_path(&plan.game_directory_name)?;
    let mods_dir = game_dir.join("mods");
    let resourcepacks_dir = game_dir.join("resourcepacks");
    let shaderpacks_dir = game_dir.join("shaderpacks");
    let config_dir = game_dir.join("config");

    fs::create_dir_all(&mods_dir).map_err(|error| {
        format!(
            "Could not create the mods folder at {}: {error}",
            mods_dir.display()
        )
    })?;
    fs::create_dir_all(&resourcepacks_dir).map_err(|error| {
        format!(
            "Could not create the resource packs folder at {}: {error}",
            resourcepacks_dir.display()
        )
    })?;
    fs::create_dir_all(&shaderpacks_dir).map_err(|error| {
        format!(
            "Could not create the shader packs folder at {}: {error}",
            shaderpacks_dir.display()
        )
    })?;
    fs::create_dir_all(&config_dir).map_err(|error| {
        format!(
            "Could not create the config folder at {}: {error}",
            config_dir.display()
        )
    })?;

    let resource_results =
        managed_resources::apply_plan_resource_actions(plan, manifest, &game_dir)?;

    let receipt_path = game_dir.join(RECEIPT_FILE_NAME);
    let receipt = SetupReceipt {
        manifest_id: &manifest.id,
        manifest_version: &manifest.manifest_version,
        server_id: &plan.server_id,
        server_name: &plan.server_name,
        server_address: &plan.server_address,
        minecraft_version: &plan.minecraft_version,
        loader_kind: plan.loader_kind,
        loader_version: plan.loader_version.as_deref(),
        game_directory_name: &plan.game_directory_name,
        launcher: format!("{:?}", plan.launcher),
        performance_profile: plan.profile.clone(),
        required_mods: &plan.required_mods,
        optional_mods: &plan.optional_mods,
    };
    let receipt_json = serde_json::to_string_pretty(&receipt)
        .map_err(|error| format!("Could not prepare the setup receipt: {error}"))?;
    atomic_file::write(&receipt_path, receipt_json.as_bytes(), "setup receipt")?;

    let mut log = vec![
        "Created the separate game folder.".to_string(),
        "Created the mods, resource packs, shader packs, and config folders.".to_string(),
        "Saved a setup receipt in the game folder.".to_string(),
    ];
    log.extend(resource_results.iter().map(|result| match result.action {
        ManagedResourceAction::Downloaded => format!(
                    "Downloaded managed resource {} to {}.",
                    result.resource_id,
                    result
                        .path
                        .as_ref()
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|| "unknown path".to_string())
                ),
        ManagedResourceAction::Verified => {
            format!("Managed resource {} was already ready.", result.resource_id)
        }
        ManagedResourceAction::Removed => format!(
                    "Removed managed resource {} at {}.",
                    result.resource_id,
                    result
                        .path
                        .as_ref()
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|| "unknown path".to_string())
                ),
        ManagedResourceAction::Missing => format!(
            "Managed resource {} was already absent.",
            result.resource_id
        ),
        ManagedResourceAction::KeptCurrent => format!(
            "Kept managed resource {} because its replacement uses the same file.",
            result.resource_id
        ),
        ManagedResourceAction::SkippedModified => format!(
            "Kept managed resource {} because it was changed outside this app.",
            result.resource_id
        ),
        ManagedResourceAction::SkippedNoFileName => format!(
            "Skipped removing managed resource {} because no managed file name is known.",
            result.resource_id
        ),
    }));

    Ok(LocalInstallResult {
        game_dir,
        receipt_path,
        resource_results,
        log,
    })
}

pub fn validate_local_install(
    plan: &InstallPlan,
    manifest: &SetupManifest,
) -> Result<LocalValidationResult, String> {
    let game_dir = game_directory_path(&plan.game_directory_name)?;
    let receipt_path = game_dir.join(RECEIPT_FILE_NAME);

    let receipt_exists = receipt_path.is_file();
    let receipt_matches = receipt_exists && receipt_matches(&receipt_path, plan, manifest)?;

    Ok(LocalValidationResult {
        game_dir_exists: game_dir.is_dir(),
        mods_dir_exists: game_dir.join("mods").is_dir(),
        receipt_exists,
        receipt_matches,
        game_dir,
        receipt_path,
    })
}

fn receipt_matches(
    path: &std::path::Path,
    plan: &InstallPlan,
    manifest: &SetupManifest,
) -> Result<bool, String> {
    let contents = fs::read(path).map_err(|error| {
        format!(
            "Could not read the setup receipt at {}: {error}",
            path.display()
        )
    })?;
    let receipt: serde_json::Value = serde_json::from_slice(&contents)
        .map_err(|error| format!("Could not read the setup receipt format: {error}"))?;

    Ok(receipt
        .get("manifestId")
        .and_then(serde_json::Value::as_str)
        == Some(manifest.id.as_str())
        && receipt
            .get("manifestVersion")
            .and_then(serde_json::Value::as_str)
            == Some(manifest.manifest_version.as_str())
        && receipt.get("serverId").and_then(serde_json::Value::as_str)
            == Some(plan.server_id.as_str())
        && receipt
            .get("performanceProfile")
            .and_then(serde_json::Value::as_str)
            == Some(plan.profile.as_str()))
}

pub fn export_install_report() -> Result<crate::commands::DiagnosticBundle, String> {
    let desktop = paths::desktop_dir()?;
    fs::create_dir_all(&desktop).map_err(|error| {
        format!(
            "Could not open the Desktop folder at {}: {error}",
            desktop.display()
        )
    })?;

    let report_path = desktop.join("minecraft-setup-manager-report.json");
    let report = serde_json::json!({
        "app": APP_SUPPORT_NAME,
        "message": "Minecraft Setup Manager created the local folder structure and setup file."
    });
    fs::write(
        &report_path,
        serde_json::to_string_pretty(&report)
            .map_err(|error| format!("Could not prepare the report: {error}"))?,
    )
    .map_err(|error| {
        format!(
            "Could not write the report at {}: {error}",
            report_path.display()
        )
    })?;

    Ok(crate::commands::DiagnosticBundle {
        path: report_path.display().to_string(),
        summary: "Saved the setup report on your Desktop.".to_string(),
    })
}

fn game_directory_path(game_directory_name: &str) -> Result<PathBuf, String> {
    let root = paths::app_support_dir(APP_SUPPORT_NAME)?;
    crate::system::path_safety::safe_child_path(&root, game_directory_name, "game folder")
}
