use reqwest::Url;
use serde::Deserialize;

use crate::commands::InstallPlan;
use crate::http_client;
use crate::manifest::schema::{
    ManifestLoaderKind, ManifestResource, ManifestResourceHashes, ManifestResourceSource,
    ManifestResourceType,
};
use crate::system::path_safety;

const MODRINTH_API_BASE_URL: &str = "https://api.modrinth.com/v2/";
const MAX_VERSION_METADATA_BYTES: u64 = 2 * 1024 * 1024;
const MAX_RESOURCE_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedModrinthFile {
    pub url: String,
    pub file_name: String,
    pub hashes: ManifestResourceHashes,
}

#[derive(Debug, Deserialize)]
struct ModrinthVersion {
    id: String,
    project_id: String,
    game_versions: Vec<String>,
    loaders: Vec<String>,
    files: Vec<ModrinthFile>,
}

#[derive(Debug, Deserialize)]
struct ModrinthProject {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ModrinthFile {
    hashes: ModrinthHashes,
    url: String,
    filename: String,
    primary: bool,
    size: u64,
}

#[derive(Debug, Deserialize)]
struct ModrinthHashes {
    sha512: String,
}

pub fn resolve_resource(
    resource: &ManifestResource,
    plan: &InstallPlan,
) -> Result<ResolvedModrinthFile, String> {
    let ManifestResourceSource::Modrinth { version, .. } = &resource.source else {
        return Err(format!(
            "Resource {} is not a Modrinth resource.",
            resource.id
        ));
    };
    let url = version_url(version)?;
    let bytes = http_client::get_bytes(
        url.as_str(),
        MAX_VERSION_METADATA_BYTES,
        "Modrinth version details",
    )?;
    let metadata: ModrinthVersion = serde_json::from_slice(&bytes)
        .map_err(|error| format!("Modrinth returned invalid version details: {error}"))?;
    let ManifestResourceSource::Modrinth { project, .. } = &resource.source else {
        return Err(format!(
            "Resource {} is not a Modrinth resource.",
            resource.id
        ));
    };
    let expected_project_id = if metadata.project_id == *project {
        project.clone()
    } else {
        resolve_project_id(project)?
    };

    resolve_version(resource, plan, metadata, &expected_project_id)
}

fn resolve_version(
    resource: &ManifestResource,
    plan: &InstallPlan,
    metadata: ModrinthVersion,
    expected_project_id: &str,
) -> Result<ResolvedModrinthFile, String> {
    let ManifestResourceSource::Modrinth { project, version } = &resource.source else {
        return Err(format!(
            "Resource {} is not a Modrinth resource.",
            resource.id
        ));
    };

    if metadata.id != *version {
        return Err(format!(
            "Modrinth returned the wrong version for resource {}.",
            resource.id
        ));
    }
    if metadata.project_id != expected_project_id {
        return Err(format!(
            "Modrinth version {} does not belong to project {}.",
            version, project
        ));
    }
    if !metadata
        .game_versions
        .iter()
        .any(|version| version == &plan.minecraft_version)
    {
        return Err(format!(
            "Modrinth resource {} does not support Minecraft {}.",
            resource.id, plan.minecraft_version
        ));
    }

    let expected_loader = match plan.loader_kind {
        ManifestLoaderKind::Fabric => "fabric",
        ManifestLoaderKind::None => "minecraft",
    };
    let loader_matches = metadata.loaders.iter().any(|loader| {
        loader == expected_loader
            || (!matches!(resource.resource_type, ManifestResourceType::Mod)
                && loader == "minecraft")
    });
    if !loader_matches {
        return Err(format!(
            "Modrinth resource {} does not support this Minecraft loader.",
            resource.id
        ));
    }

    let file = metadata
        .files
        .iter()
        .find(|file| file.primary)
        .or_else(|| metadata.files.first())
        .ok_or_else(|| format!("Modrinth version {version} does not contain a download file."))?;
    if file.size > MAX_RESOURCE_BYTES {
        return Err(format!("Modrinth resource {} is too large.", resource.id));
    }
    validate_cdn_url(&file.url)?;
    validate_sha512(&file.hashes.sha512, &resource.id)?;
    if let Some(expected) = &resource.hashes.sha512 {
        if !expected.eq_ignore_ascii_case(&file.hashes.sha512) {
            return Err(format!(
                "Modrinth resource {} does not match the hash pinned by the server.",
                resource.id
            ));
        }
    }

    let file_name = resource
        .file_name
        .clone()
        .unwrap_or_else(|| file.filename.clone());
    path_safety::validate_portable_component(&file_name, "Modrinth file name")?;
    let hashes = ManifestResourceHashes {
        sha512: Some(file.hashes.sha512.to_ascii_lowercase()),
        sha256: resource.hashes.sha256.clone(),
    };

    Ok(ResolvedModrinthFile {
        url: file.url.clone(),
        file_name,
        hashes,
    })
}

fn resolve_project_id(project: &str) -> Result<String, String> {
    let url = project_url(project)?;
    let bytes = http_client::get_bytes(
        url.as_str(),
        MAX_VERSION_METADATA_BYTES,
        "Modrinth project details",
    )?;
    let metadata: ModrinthProject = serde_json::from_slice(&bytes)
        .map_err(|error| format!("Modrinth returned invalid project details: {error}"))?;
    Ok(metadata.id)
}

fn version_url(version: &str) -> Result<Url, String> {
    let mut url = Url::parse(MODRINTH_API_BASE_URL)
        .map_err(|error| format!("Could not prepare the Modrinth API URL: {error}"))?;
    url.path_segments_mut()
        .map_err(|_| "Could not prepare the Modrinth API URL.".to_string())?
        .extend(["version", version]);
    Ok(url)
}

fn project_url(project: &str) -> Result<Url, String> {
    let mut url = Url::parse(MODRINTH_API_BASE_URL)
        .map_err(|error| format!("Could not prepare the Modrinth API URL: {error}"))?;
    url.path_segments_mut()
        .map_err(|_| "Could not prepare the Modrinth API URL.".to_string())?
        .extend(["project", project]);
    Ok(url)
}

fn validate_cdn_url(url: &str) -> Result<(), String> {
    let parsed =
        Url::parse(url).map_err(|_| "Modrinth returned an invalid file URL.".to_string())?;
    if parsed.scheme() == "https" && parsed.host_str() == Some("cdn.modrinth.com") {
        Ok(())
    } else {
        Err("Modrinth returned a file URL outside its secure CDN.".to_string())
    }
}

fn validate_sha512(hash: &str, resource_id: &str) -> Result<(), String> {
    if hash.len() == 128 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(format!(
            "Modrinth returned an invalid SHA-512 hash for resource {resource_id}."
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{LauncherKind, ServerUpdateStatus};
    use crate::manifest::schema::{ManifestResourceTarget, ManifestResourceType};

    #[test]
    fn resolves_the_primary_compatible_file() {
        let resolved =
            resolve_version(&resource(), &plan(), version(), "project").expect("resolve file");

        assert_eq!(resolved.file_name, "primary.jar");
        assert_eq!(
            resolved.url,
            "https://cdn.modrinth.com/data/project/primary.jar"
        );
        assert_eq!(resolved.hashes.sha512, Some("a".repeat(128)));
    }

    #[test]
    fn rejects_a_version_from_another_project() {
        let mut metadata = version();
        metadata.project_id = "another".to_string();

        let error = resolve_version(&resource(), &plan(), metadata, "project")
            .expect_err("wrong project must fail");

        assert!(error.contains("does not belong"));
    }

    #[test]
    fn rejects_incompatible_minecraft_versions() {
        let mut metadata = version();
        metadata.game_versions = vec!["1.20.1".to_string()];

        let error = resolve_version(&resource(), &plan(), metadata, "project")
            .expect_err("wrong game version must fail");

        assert!(error.contains("does not support Minecraft"));
    }

    #[test]
    fn rejects_downloads_outside_the_modrinth_cdn() {
        let mut metadata = version();
        metadata.files[0].url = "https://example.com/file.jar".to_string();

        let error = resolve_version(&resource(), &plan(), metadata, "project")
            .expect_err("foreign CDN must fail");

        assert!(error.contains("secure CDN"));
    }

    fn resource() -> ManifestResource {
        ManifestResource {
            id: "example-mod".to_string(),
            name: "Example Mod".to_string(),
            resource_type: ManifestResourceType::Mod,
            target: ManifestResourceTarget::Mods,
            required: true,
            profiles: vec![],
            file_name: None,
            source: ManifestResourceSource::Modrinth {
                project: "project".to_string(),
                version: "version".to_string(),
            },
            hashes: ManifestResourceHashes::default(),
        }
    }

    fn plan() -> InstallPlan {
        InstallPlan {
            server_id: "example".to_string(),
            update_status: ServerUpdateStatus::NewSetup,
            minecraft_version: "1.21.6".to_string(),
            loader_kind: ManifestLoaderKind::Fabric,
            loader_version: Some("0.16.14".to_string()),
            game_directory_name: "Example".to_string(),
            server_name: "Example".to_string(),
            server_address: "play.example.com".to_string(),
            launcher: LauncherKind::Official,
            profile: "balanced".to_string(),
            profile_label: "Balanced".to_string(),
            recommended_memory_mb: 4096,
            actions: vec![],
            required_mods: vec![],
            optional_mods: vec![],
            warnings: vec![],
        }
    }

    fn version() -> ModrinthVersion {
        ModrinthVersion {
            id: "version".to_string(),
            project_id: "project".to_string(),
            game_versions: vec!["1.21.6".to_string()],
            loaders: vec!["fabric".to_string()],
            files: vec![
                ModrinthFile {
                    hashes: ModrinthHashes {
                        sha512: "a".repeat(128),
                    },
                    url: "https://cdn.modrinth.com/data/project/primary.jar".to_string(),
                    filename: "primary.jar".to_string(),
                    primary: true,
                    size: 1024,
                },
                ModrinthFile {
                    hashes: ModrinthHashes {
                        sha512: "b".repeat(128),
                    },
                    url: "https://cdn.modrinth.com/data/project/secondary.jar".to_string(),
                    filename: "secondary.jar".to_string(),
                    primary: false,
                    size: 1024,
                },
            ],
        }
    }
}
