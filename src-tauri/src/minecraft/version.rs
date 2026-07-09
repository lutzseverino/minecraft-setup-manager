use crate::commands::InstallPlan;
use crate::manifest::schema::ManifestLoaderKind;

pub fn installed_version_id(plan: &InstallPlan) -> String {
    match plan.loader_kind {
        ManifestLoaderKind::None => plan.minecraft_version.clone(),
        ManifestLoaderKind::Fabric => format!(
            "fabric-loader-{}-{}",
            plan.loader_version.as_deref().unwrap_or("unknown"),
            plan.minecraft_version
        ),
    }
}

pub fn version_label(plan: &InstallPlan) -> String {
    match plan.loader_kind {
        ManifestLoaderKind::None => format!("Minecraft {}", plan.minecraft_version),
        ManifestLoaderKind::Fabric => format!(
            "Fabric {} for Minecraft {}",
            plan.loader_version.as_deref().unwrap_or("unknown"),
            plan.minecraft_version
        ),
    }
}
