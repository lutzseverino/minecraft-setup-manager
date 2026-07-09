use crate::commands::{InstallPlan, ValidationCheck, ValidationResult, ValidationStatus};
use crate::setup::ClientSetupValidation;

pub fn validate_client_setup(
    plan: &InstallPlan,
    validation: &ClientSetupValidation,
) -> ValidationResult {
    let local_install = &validation.local_install;
    let launcher_profile = &validation.launcher_profile;
    let server_entry = &validation.server_entry;
    let mut checks = vec![
        ValidationCheck {
            id: "manifest".to_string(),
            label: "Setup list loaded".to_string(),
            detail: format!(
                "Minecraft {}, {}, {} required mods, and {} extra mods.",
                plan.minecraft_version,
                loader_summary(plan),
                plan.required_mods.len(),
                plan.optional_mods.len()
            ),
            status: ValidationStatus::Pass,
        },
        ValidationCheck {
            id: "loader_version".to_string(),
            label: "Minecraft version".to_string(),
            detail: if launcher_profile.version_exists {
                format!(
                    "Version {} is installed.",
                    launcher_profile.expected_version_id
                )
            } else {
                format!(
                    "Version {} is missing.",
                    launcher_profile.expected_version_id
                )
            },
            status: status_from_bool(launcher_profile.version_exists),
        },
        ValidationCheck {
            id: "game_directory".to_string(),
            label: "Separate game folder".to_string(),
            detail: if local_install.game_dir_exists {
                format!("Folder is ready at {}.", local_install.game_dir.display())
            } else {
                format!("Folder is missing at {}.", local_install.game_dir.display())
            },
            status: status_from_bool(local_install.game_dir_exists),
        },
        ValidationCheck {
            id: "launcher_profile".to_string(),
            label: "Launcher profile".to_string(),
            detail: launcher_profile_detail(launcher_profile),
            status: launcher_profile_status(launcher_profile),
        },
        ValidationCheck {
            id: "mods_directory".to_string(),
            label: "Mods folder".to_string(),
            detail: if local_install.mods_dir_exists {
                "Mods folder is ready.".to_string()
            } else {
                "Mods folder is missing.".to_string()
            },
            status: status_from_bool(local_install.mods_dir_exists),
        },
        ValidationCheck {
            id: "setup_receipt".to_string(),
            label: "Setup file".to_string(),
            detail: if local_install.receipt_exists {
                format!(
                    "Setup file is saved at {}.",
                    local_install.receipt_path.display()
                )
            } else {
                format!(
                    "Setup file is missing at {}.",
                    local_install.receipt_path.display()
                )
            },
            status: status_from_bool(local_install.receipt_exists),
        },
    ];
    if server_entry.required {
        checks.push(ValidationCheck {
            id: "server_entry".to_string(),
            label: "Saved server".to_string(),
            detail: if server_entry.entry_matches {
                format!("Server is saved in {}.", server_entry.path.display())
            } else if server_entry.file_exists {
                "The server list exists, but this server entry is missing or different.".to_string()
            } else {
                format!("Server list is missing at {}.", server_entry.path.display())
            },
            status: status_from_bool(server_entry.entry_matches),
        });
    }
    let overall = overall_status(&checks);

    ValidationResult { overall, checks }
}

fn loader_summary(plan: &InstallPlan) -> String {
    match plan.loader_kind {
        crate::manifest::schema::ManifestLoaderKind::None => "no extra loader".to_string(),
        crate::manifest::schema::ManifestLoaderKind::Fabric => format!(
            "Fabric {}",
            plan.loader_version.as_deref().unwrap_or("unknown")
        ),
    }
}

fn launcher_profile_detail(profile: &crate::launcher::LauncherProfileValidation) -> String {
    if !profile.required {
        return "Manual setup selected; launcher profile changes are skipped.".to_string();
    }

    if !profile.launcher_profiles_exists {
        return profile
            .launcher_profiles_path
            .as_ref()
            .map(|path| format!("Launcher profiles file is missing at {}.", path.display()))
            .unwrap_or_else(|| "Launcher profiles file is missing.".to_string());
    }

    if !profile.profile_exists {
        return format!("Profile {} is missing.", profile.profile_id);
    }

    if !profile.game_dir_matches {
        return format!(
            "Profile does not point at {}.",
            profile.expected_game_dir.display()
        );
    }

    if !profile.version_matches {
        return format!("Profile does not use {}.", profile.expected_version_id);
    }

    "Launcher profile is ready.".to_string()
}

fn launcher_profile_status(
    profile: &crate::launcher::LauncherProfileValidation,
) -> ValidationStatus {
    if !profile.required {
        return ValidationStatus::Warning;
    }

    status_from_bool(
        profile.launcher_profiles_exists
            && profile.profile_exists
            && profile.game_dir_matches
            && profile.version_matches,
    )
}

fn status_from_bool(value: bool) -> ValidationStatus {
    if value {
        ValidationStatus::Pass
    } else {
        ValidationStatus::Fail
    }
}

fn overall_status(checks: &[ValidationCheck]) -> ValidationStatus {
    if checks
        .iter()
        .any(|check| matches!(check.status, ValidationStatus::Fail))
    {
        return ValidationStatus::Fail;
    }

    if checks
        .iter()
        .any(|check| matches!(check.status, ValidationStatus::Warning))
    {
        return ValidationStatus::Warning;
    }

    ValidationStatus::Pass
}
