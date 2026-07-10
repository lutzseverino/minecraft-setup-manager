use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use reqwest::Url;
use unicode_normalization::UnicodeNormalization;

use super::schema::{
    ManifestLoaderKind, ManifestResource, ManifestResourceSource, ManifestResourceTarget,
    ManifestResourceType, SetupManifest,
};
use crate::system::path_safety;

const SUPPORTED_SCHEMA_VERSION: u16 = 1;
const MAX_PROFILES: usize = 16;
const MAX_RESOURCES: usize = 512;
const MAX_ID_LENGTH: usize = 80;
const MAX_LABEL_LENGTH: usize = 160;

pub fn validate_manifest(manifest: &SetupManifest, source_url: &str) -> Result<(), String> {
    if manifest.schema_version != SUPPORTED_SCHEMA_VERSION {
        return Err(format!(
            "This setup file uses version {}. This app supports version {SUPPORTED_SCHEMA_VERSION}.",
            manifest.schema_version
        ));
    }

    validate_id(&manifest.id, "setup ID")?;
    validate_text(&manifest.manifest_version, "setup version", MAX_ID_LENGTH)?;
    validate_text(&manifest.display_name, "server name", MAX_LABEL_LENGTH)?;
    validate_text(&manifest.server.name, "server name", MAX_LABEL_LENGTH)?;
    validate_server_address(&manifest.server.address, "server address")?;
    validate_text(
        &manifest.minecraft.version,
        "Minecraft version",
        MAX_ID_LENGTH,
    )?;
    validate_text(
        &manifest.install.launcher_profile_name,
        "launcher profile name",
        MAX_LABEL_LENGTH,
    )?;
    validate_text(
        &manifest.install.game_directory_name,
        "game folder name",
        MAX_ID_LENGTH,
    )?;
    path_safety::validate_portable_component(
        &manifest.install.game_directory_name,
        "game folder name",
    )?;

    match manifest.minecraft.loader.kind {
        ManifestLoaderKind::None if manifest.minecraft.loader.version.is_some() => {
            return Err("A setup without a loader must not include a loader version.".to_string());
        }
        ManifestLoaderKind::Fabric => validate_text(
            manifest
                .minecraft
                .loader
                .version
                .as_deref()
                .unwrap_or_default(),
            "Fabric version",
            MAX_ID_LENGTH,
        )?,
        ManifestLoaderKind::None => {}
    }

    if manifest.profiles.is_empty() {
        return Err("The server setup must provide at least one setup option.".to_string());
    }
    if manifest.profiles.len() > MAX_PROFILES {
        return Err(format!(
            "The server setup has too many setup options. The limit is {MAX_PROFILES}."
        ));
    }
    if manifest.resources.len() > MAX_RESOURCES {
        return Err(format!(
            "The server setup has too many files. The limit is {MAX_RESOURCES}."
        ));
    }

    let mut profile_ids = HashSet::new();
    for profile in &manifest.profiles {
        validate_id(&profile.id, "setup option ID")?;
        validate_text(&profile.label, "setup option name", MAX_LABEL_LENGTH)?;
        if !(512..=65_536).contains(&profile.recommended_memory_mb) {
            return Err(format!(
                "Setup option {} must recommend between 512 MB and 65536 MB of memory.",
                profile.id
            ));
        }
        if !profile_ids.insert(profile.id.as_str()) {
            return Err(format!(
                "Setup option ID {} is used more than once.",
                profile.id
            ));
        }
    }

    let source_is_loopback = is_loopback_url(source_url)?;
    validate_resources(manifest, &profile_ids, source_is_loopback)?;

    if let Some(entry) = &manifest.server_entry {
        validate_text(&entry.name, "saved server name", MAX_LABEL_LENGTH)?;
        validate_server_address(&entry.address, "saved server address")?;
    }

    Ok(())
}

fn validate_resources(
    manifest: &SetupManifest,
    profile_ids: &HashSet<&str>,
    source_is_loopback: bool,
) -> Result<(), String> {
    let mut resource_ids = HashSet::new();
    let mut destinations: HashMap<(ManifestResourceTarget, String), &str> = HashMap::new();

    for resource in &manifest.resources {
        validate_id(&resource.id, "resource ID")?;
        validate_text(&resource.name, "resource name", MAX_LABEL_LENGTH)?;
        if !resource_ids.insert(resource.id.as_str()) {
            return Err(format!(
                "Resource ID {} is used more than once.",
                resource.id
            ));
        }

        validate_resource_target(resource)?;
        if matches!(resource.resource_type, ManifestResourceType::Mod)
            && matches!(manifest.minecraft.loader.kind, ManifestLoaderKind::None)
        {
            return Err(format!(
                "Resource {} is a mod, but this setup does not choose a mod loader.",
                resource.id
            ));
        }
        validate_resource_profiles(resource, profile_ids)?;
        validate_resource_source(resource, source_is_loopback)?;

        let file_name = resource
            .file_name
            .as_deref()
            .ok_or_else(|| format!("Resource {} must provide its exact file name.", resource.id))?;
        path_safety::validate_portable_component(file_name, "resource file name")?;
        let destination = (resource.target.clone(), file_name.to_lowercase());
        if let Some(existing_id) = destinations.insert(destination, resource.id.as_str()) {
            return Err(format!(
                "Resources {existing_id} and {} use the same destination file.",
                resource.id
            ));
        }
    }

    Ok(())
}

fn validate_resource_target(resource: &ManifestResource) -> Result<(), String> {
    let matches = matches!(
        (&resource.resource_type, &resource.target),
        (ManifestResourceType::Mod, ManifestResourceTarget::Mods)
            | (
                ManifestResourceType::ResourcePack,
                ManifestResourceTarget::Resourcepacks
            )
            | (
                ManifestResourceType::ShaderPack,
                ManifestResourceTarget::Shaderpacks
            )
            | (ManifestResourceType::Config, ManifestResourceTarget::Config)
    );

    if matches {
        Ok(())
    } else {
        Err(format!(
            "Resource {} uses a folder that does not match its type.",
            resource.id
        ))
    }
}

fn validate_resource_profiles(
    resource: &ManifestResource,
    profile_ids: &HashSet<&str>,
) -> Result<(), String> {
    let mut seen = HashSet::new();
    for profile in &resource.profiles {
        if !profile_ids.contains(profile.as_str()) {
            return Err(format!(
                "Resource {} refers to unknown setup option {}.",
                resource.id, profile
            ));
        }
        if !seen.insert(profile) {
            return Err(format!(
                "Resource {} lists setup option {} more than once.",
                resource.id, profile
            ));
        }
    }
    Ok(())
}

fn validate_resource_source(
    resource: &ManifestResource,
    source_is_loopback: bool,
) -> Result<(), String> {
    match &resource.source {
        ManifestResourceSource::Direct { url } => {
            validate_direct_url(url, source_is_loopback)?;
            if resource.hashes.sha512.is_none() && resource.hashes.sha256.is_none() {
                return Err(format!(
                    "Direct resource {} must include a SHA-256 or SHA-512 hash.",
                    resource.id
                ));
            }
        }
        ManifestResourceSource::Modrinth { project, version } => {
            validate_id(project, "Modrinth project")?;
            validate_id(version, "Modrinth version")?;
        }
    }

    if let Some(hash) = &resource.hashes.sha512 {
        validate_hash(hash, 128, "SHA-512", &resource.id)?;
    }
    if let Some(hash) = &resource.hashes.sha256 {
        validate_hash(hash, 64, "SHA-256", &resource.id)?;
    }
    if let Some(file_name) = &resource.file_name {
        path_safety::validate_portable_component(file_name, "resource file name")?;
    }

    Ok(())
}

fn validate_direct_url(url: &str, source_is_loopback: bool) -> Result<(), String> {
    let parsed = Url::parse(url).map_err(|_| "A direct resource URL is not valid.".to_string())?;
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("Direct resource URLs must not contain a username or password.".to_string());
    }
    let loopback = is_loopback_parsed_url(&parsed);
    let non_public = is_non_public_parsed_url(&parsed);

    if parsed.scheme() == "https" && !non_public {
        return Ok(());
    }
    if source_is_loopback && loopback && matches!(parsed.scheme(), "http" | "https") {
        return Ok(());
    }

    Err("Direct resource URLs must use public HTTPS. Local URLs are only allowed for a local setup file.".to_string())
}

fn validate_hash(hash: &str, length: usize, kind: &str, resource_id: &str) -> Result<(), String> {
    if hash.len() == length
        && hash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        Ok(())
    } else {
        Err(format!(
            "Resource {resource_id} has an invalid {kind} hash."
        ))
    }
}

fn validate_id(value: &str, label: &str) -> Result<(), String> {
    validate_text(value, label, MAX_ID_LENGTH)?;
    if value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        Ok(())
    } else {
        Err(format!(
            "The {label} may only use letters, numbers, dots, dashes, and underscores."
        ))
    }
}

fn validate_text(value: &str, label: &str, max_length: usize) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("The {label} must not be empty."));
    }
    if value.chars().count() > max_length {
        return Err(format!("The {label} is too long."));
    }
    if value.chars().any(char::is_control) {
        return Err(format!(
            "The {label} contains unsupported control characters."
        ));
    }
    if value != value.nfc().collect::<String>() {
        return Err(format!("The {label} must use normalized Unicode text."));
    }
    Ok(())
}

fn validate_server_address(value: &str, label: &str) -> Result<(), String> {
    validate_text(value, label, MAX_LABEL_LENGTH)?;
    if value.contains("://") || value.contains('/') || value.chars().any(char::is_whitespace) {
        return Err(format!(
            "The {label} must look like play.example.com or play.example.com:25565."
        ));
    }
    Ok(())
}

fn is_loopback_url(url: &str) -> Result<bool, String> {
    let parsed = Url::parse(url).map_err(|_| "The setup file URL is not valid.".to_string())?;
    Ok(is_loopback_parsed_url(&parsed))
}

fn is_loopback_parsed_url(url: &Url) -> bool {
    let Some(host) = url.host_str() else {
        return false;
    };
    let host = host.trim_start_matches('[').trim_end_matches(']');
    if host.eq_ignore_ascii_case("localhost") || host.ends_with(".localhost") {
        return true;
    }

    host.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())
}

fn is_non_public_parsed_url(url: &Url) -> bool {
    let Some(host) = url.host_str() else {
        return true;
    };
    let host = host.trim_start_matches('[').trim_end_matches(']');
    host.eq_ignore_ascii_case("localhost")
        || host.ends_with(".localhost")
        || host.ends_with(".local")
        || host.parse::<IpAddr>().is_ok_and(is_non_public_ip)
}

pub(crate) fn is_non_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_non_public_ipv4(ip),
        IpAddr::V6(ip) => is_non_public_ipv6(ip),
    }
}

fn is_non_public_ipv4(ip: Ipv4Addr) -> bool {
    ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.is_unspecified()
        || ip.is_multicast()
}

fn is_non_public_ipv6(ip: Ipv6Addr) -> bool {
    ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_unique_local()
        || ip.is_unicast_link_local()
        || ip.is_multicast()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::schema::{
        ManifestInstall, ManifestLoader, ManifestMinecraft, ManifestPerformanceProfile,
        ManifestResourceHashes, ManifestServer,
    };

    #[test]
    fn accepts_a_valid_manifest() {
        assert!(validate_manifest(&manifest(), "https://setup.example.com/manifest.json").is_ok());
    }

    #[test]
    fn rejects_a_game_directory_that_can_escape_the_managed_root() {
        let mut manifest = manifest();
        manifest.install.game_directory_name = "../outside".to_string();

        let error = validate_manifest(&manifest, "https://setup.example.com/manifest.json")
            .expect_err("unsafe path must fail");

        assert!(error.contains("game folder name"));
    }

    #[test]
    fn rejects_duplicate_destinations_for_overlapping_profiles() {
        let mut manifest = manifest();
        let mut duplicate = manifest.resources[0].clone();
        duplicate.id = "second".to_string();
        manifest.resources.push(duplicate);

        let error = validate_manifest(&manifest, "https://setup.example.com/manifest.json")
            .expect_err("duplicate destination must fail");

        assert!(error.contains("same destination file"));
    }

    #[test]
    fn rejects_duplicate_destinations_for_disjoint_profiles() {
        let mut manifest = manifest();
        manifest.resources[0].profiles = vec!["light".to_string()];
        let mut alternative = manifest.resources[0].clone();
        alternative.id = "visual-file".to_string();
        alternative.profiles = vec!["visual".to_string()];
        manifest.resources.push(alternative);

        let error = validate_manifest(&manifest, "https://setup.example.com/manifest.json")
            .expect_err("profile changes must not transfer file ownership");

        assert!(error.contains("same destination file"));
    }

    #[test]
    fn remote_manifests_cannot_request_local_urls() {
        let mut manifest = manifest();
        manifest.resources[0].source = ManifestResourceSource::Direct {
            url: "http://127.0.0.1:8080/file.jar".to_string(),
        };

        let error = validate_manifest(&manifest, "https://setup.example.com/manifest.json")
            .expect_err("remote-to-local request must fail");

        assert!(error.contains("public HTTPS"));
    }

    #[test]
    fn local_manifests_can_use_local_urls_for_development() {
        let mut manifest = manifest();
        manifest.resources[0].source = ManifestResourceSource::Direct {
            url: "http://127.0.0.1:8080/file.jar".to_string(),
        };

        assert!(validate_manifest(&manifest, "http://127.0.0.1:8080/manifest.json").is_ok());
    }

    #[test]
    fn rejects_invalid_hashes() {
        let mut manifest = manifest();
        manifest.resources[0].hashes.sha256 = Some("not-a-hash".to_string());

        let error = validate_manifest(&manifest, "https://setup.example.com/manifest.json")
            .expect_err("invalid hash must fail");

        assert!(error.contains("invalid SHA-256"));
    }

    #[test]
    fn schema_rejects_unknown_fields() {
        let mut value = serde_json::to_value(manifest()).expect("serialize manifest");
        value
            .as_object_mut()
            .expect("manifest object")
            .insert("runThis".to_string(), serde_json::json!("anything"));

        let error = serde_json::from_value::<SetupManifest>(value)
            .expect_err("unknown field must fail")
            .to_string();

        assert!(error.contains("unknown field"));
    }

    fn manifest() -> SetupManifest {
        SetupManifest {
            schema_version: 1,
            manifest_version: "1".to_string(),
            id: "example".to_string(),
            display_name: "Example Server".to_string(),
            server: ManifestServer {
                name: "Example Server".to_string(),
                address: "play.example.com".to_string(),
            },
            minecraft: ManifestMinecraft {
                version: "1.21.6".to_string(),
                loader: ManifestLoader {
                    kind: ManifestLoaderKind::Fabric,
                    version: Some("0.16.14".to_string()),
                },
            },
            install: ManifestInstall {
                game_directory_name: "Example Server".to_string(),
                launcher_profile_name: "Example Server".to_string(),
            },
            profiles: vec![
                ManifestPerformanceProfile {
                    id: "light".to_string(),
                    label: "Light".to_string(),
                    recommended_memory_mb: 3072,
                    includes_shaders: false,
                },
                ManifestPerformanceProfile {
                    id: "visual".to_string(),
                    label: "Visual".to_string(),
                    recommended_memory_mb: 6144,
                    includes_shaders: true,
                },
            ],
            resources: vec![ManifestResource {
                id: "fabric-api".to_string(),
                name: "Fabric API".to_string(),
                resource_type: ManifestResourceType::Mod,
                target: ManifestResourceTarget::Mods,
                required: true,
                profiles: vec![],
                file_name: Some("fabric-api.jar".to_string()),
                source: ManifestResourceSource::Direct {
                    url: "https://cdn.example.com/fabric-api.jar".to_string(),
                },
                hashes: ManifestResourceHashes {
                    sha512: None,
                    sha256: Some("00".repeat(32)),
                },
            }],
            server_entry: None,
        }
    }
}
