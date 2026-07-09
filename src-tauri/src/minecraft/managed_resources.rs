use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256, Sha512};

use crate::commands::{
    InstallPlan, SetupActionIntent, SetupActionKind, SetupActionStatus, SetupActionTarget,
};
use crate::http_client;
use crate::manifest::schema::{
    ManifestResource, ManifestResourceHashes, ManifestResourceSource, ManifestResourceTarget,
    SetupManifest,
};
use crate::system::atomic_file;

const MAX_RESOURCE_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagedResourceAction {
    Downloaded,
    Verified,
    Removed,
    Missing,
    SkippedNoFileName,
    SkippedUnsupportedSource,
    SkippedMissingHash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedResourceResult {
    pub resource_id: String,
    pub action: ManagedResourceAction,
    pub path: Option<PathBuf>,
}

pub fn apply_plan_resource_actions(
    plan: &InstallPlan,
    manifest: &SetupManifest,
    game_dir: &Path,
) -> Result<Vec<ManagedResourceResult>, String> {
    let mut results = sync_direct_resources(plan, manifest, game_dir)?;
    results.extend(remove_stale_resources(plan, game_dir)?);

    Ok(results)
}

fn sync_direct_resources(
    plan: &InstallPlan,
    manifest: &SetupManifest,
    game_dir: &Path,
) -> Result<Vec<ManagedResourceResult>, String> {
    manifest
        .resources
        .iter()
        .filter(|resource| {
            plan.actions.iter().any(|action| {
                matches!(action.kind, SetupActionKind::SyncResource)
                    && matches!(
                        action.intent,
                        SetupActionIntent::Add
                            | SetupActionIntent::Update
                            | SetupActionIntent::Verify
                    )
                    && matches!(action.status, SetupActionStatus::Ready)
                    && action.resource_id.as_deref() == Some(resource.id.as_str())
            })
        })
        .map(|resource| sync_direct_resource(resource, game_dir))
        .collect()
}

fn sync_direct_resource(
    resource: &ManifestResource,
    game_dir: &Path,
) -> Result<ManagedResourceResult, String> {
    let resource_id = resource.id.clone();
    let Some(file_name) = managed_file_name(resource) else {
        return Ok(ManagedResourceResult {
            resource_id,
            action: ManagedResourceAction::SkippedNoFileName,
            path: None,
        });
    };
    let ManifestResourceSource::Direct { url } = &resource.source else {
        return Ok(ManagedResourceResult {
            resource_id,
            action: ManagedResourceAction::SkippedUnsupportedSource,
            path: None,
        });
    };
    if !has_expected_hash(&resource.hashes) {
        return Ok(ManagedResourceResult {
            resource_id,
            action: ManagedResourceAction::SkippedMissingHash,
            path: None,
        });
    }

    let target = setup_action_target(resource.target.clone());
    let path = managed_file_path(game_dir, target, &file_name)?;

    if path.is_file() && file_matches_hashes(&path, &resource.hashes)? {
        return Ok(ManagedResourceResult {
            resource_id,
            action: ManagedResourceAction::Verified,
            path: Some(path),
        });
    }

    let parent = path
        .parent()
        .ok_or_else(|| format!("Managed resource path {} has no parent.", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "Could not create managed resource folder at {}: {error}",
            parent.display()
        )
    })?;
    let temp_path = path.with_file_name(format!(".minecraft-setup-manager-{}.tmp", file_name));
    http_client::download_to_path(url, &temp_path, MAX_RESOURCE_BYTES, "setup resource")?;

    if !file_matches_hashes(&temp_path, &resource.hashes)? {
        let _ = fs::remove_file(&temp_path);
        return Err(format!(
            "Downloaded resource {} did not match its expected hash.",
            resource.id
        ));
    }

    atomic_file::replace_file(
        &temp_path,
        &path,
        &format!("managed resource {}", resource.id),
    )?;

    Ok(ManagedResourceResult {
        resource_id,
        action: ManagedResourceAction::Downloaded,
        path: Some(path),
    })
}

fn remove_stale_resources(
    plan: &InstallPlan,
    game_dir: &Path,
) -> Result<Vec<ManagedResourceResult>, String> {
    plan.actions
        .iter()
        .filter(|action| matches!(action.kind, SetupActionKind::RemoveResource))
        .filter(|action| matches!(action.status, SetupActionStatus::Ready))
        .map(|action| {
            let resource_id = action.resource_id.clone().ok_or_else(|| {
                "A managed resource removal is missing its resource id.".to_string()
            })?;
            let Some(file_name) = &action.file_name else {
                return Ok(ManagedResourceResult {
                    resource_id,
                    action: ManagedResourceAction::SkippedNoFileName,
                    path: None,
                });
            };
            let target = action
                .target
                .ok_or_else(|| format!("Managed resource {resource_id} is missing its target."))?;
            let path = managed_file_path(game_dir, target, file_name)?;

            if !path.is_file() {
                return Ok(ManagedResourceResult {
                    resource_id,
                    action: ManagedResourceAction::Missing,
                    path: Some(path),
                });
            }

            fs::remove_file(&path).map_err(|error| {
                format!(
                    "Could not remove managed resource {} at {}: {error}",
                    resource_id,
                    path.display()
                )
            })?;

            Ok(ManagedResourceResult {
                resource_id,
                action: ManagedResourceAction::Removed,
                path: Some(path),
            })
        })
        .collect()
}

fn has_expected_hash(hashes: &ManifestResourceHashes) -> bool {
    hashes.sha512.is_some() || hashes.sha256.is_some()
}

fn file_matches_hashes(path: &Path, hashes: &ManifestResourceHashes) -> Result<bool, String> {
    if let Some(expected) = &hashes.sha512 {
        return Ok(file_digest::<Sha512>(path)? == expected.to_ascii_lowercase());
    }

    if let Some(expected) = &hashes.sha256 {
        return Ok(file_digest::<Sha256>(path)? == expected.to_ascii_lowercase());
    }

    Ok(false)
}

fn file_digest<D>(path: &Path) -> Result<String, String>
where
    D: Digest + Default,
{
    let mut file = fs::File::open(path).map_err(|error| {
        format!(
            "Could not read managed resource at {}: {error}",
            path.display()
        )
    })?;
    let mut digest = D::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = file.read(&mut buffer).map_err(|error| {
            format!(
                "Could not read managed resource at {}: {error}",
                path.display()
            )
        })?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }

    Ok(hex_digest(&digest.finalize()))
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }

    output
}

pub fn managed_file_name(resource: &ManifestResource) -> Option<String> {
    resource
        .file_name
        .clone()
        .or_else(|| direct_source_file_name(&resource.source))
        .filter(|file_name| is_safe_file_name(file_name))
}

pub fn can_sync_resource(resource: &ManifestResource) -> bool {
    matches!(resource.source, ManifestResourceSource::Direct { .. })
        && managed_file_name(resource).is_some()
        && has_expected_hash(&resource.hashes)
}

fn direct_source_file_name(source: &ManifestResourceSource) -> Option<String> {
    let ManifestResourceSource::Direct { url } = source else {
        return None;
    };

    let parsed = reqwest::Url::parse(url).ok()?;
    parsed
        .path_segments()
        .and_then(Iterator::last)
        .filter(|segment| !segment.is_empty())
        .map(ToString::to_string)
}

fn managed_file_path(
    game_dir: &Path,
    target: SetupActionTarget,
    file_name: &str,
) -> Result<PathBuf, String> {
    crate::system::path_safety::validate_portable_component(
        file_name,
        "managed resource file name",
    )?;
    let target_dir = game_dir.join(target_dir_name(target));
    crate::system::path_safety::reject_symlink(&target_dir, "managed resource folder")?;
    Ok(target_dir.join(file_name))
}

fn is_safe_file_name(file_name: &str) -> bool {
    crate::system::path_safety::validate_portable_component(file_name, "managed resource file name")
        .is_ok()
}

fn target_dir_name(target: SetupActionTarget) -> &'static str {
    match target {
        SetupActionTarget::Mods => "mods",
        SetupActionTarget::Resourcepacks => "resourcepacks",
        SetupActionTarget::Shaderpacks => "shaderpacks",
        SetupActionTarget::Config => "config",
    }
}

pub fn setup_action_target(target: ManifestResourceTarget) -> SetupActionTarget {
    match target {
        ManifestResourceTarget::Mods => SetupActionTarget::Mods,
        ManifestResourceTarget::Resourcepacks => SetupActionTarget::Resourcepacks,
        ManifestResourceTarget::Shaderpacks => SetupActionTarget::Shaderpacks,
        ManifestResourceTarget::Config => SetupActionTarget::Config,
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::commands::{
        InstallPlan, LauncherKind, ServerUpdateStatus, SetupActionIntent, SetupActionPreview,
        SetupActionStatus,
    };
    use crate::manifest::schema::{
        ManifestInstall, ManifestLoader, ManifestLoaderKind, ManifestMinecraft, ManifestResource,
        ManifestResourceHashes, ManifestResourceSource, ManifestResourceTarget,
        ManifestResourceType, ManifestServer,
    };

    use super::*;

    #[test]
    fn applies_stale_resource_removal_inside_managed_target_dir() {
        let game_dir = test_dir("remove-managed-resource");
        let mods_dir = game_dir.join("mods");
        fs::create_dir_all(&mods_dir).expect("create mods dir");
        let file_path = mods_dir.join("old.jar");
        fs::write(&file_path, "old").expect("write managed file");

        let results = apply_plan_resource_actions(
            &plan_with_removal(Some("old.jar")),
            &empty_manifest(),
            &game_dir,
        )
        .expect("apply resource actions");

        assert!(!file_path.exists());
        assert_eq!(
            results,
            vec![ManagedResourceResult {
                resource_id: "old-mod".to_string(),
                action: ManagedResourceAction::Removed,
                path: Some(file_path)
            }]
        );
    }

    #[test]
    fn rejects_path_like_resource_file_names() {
        let game_dir = test_dir("reject-path-resource");

        let error = apply_plan_resource_actions(
            &plan_with_removal(Some("../outside.jar")),
            &empty_manifest(),
            &game_dir,
        )
        .expect_err("expected unsafe path error");

        assert!(error.contains("managed resource file name"));
    }

    #[test]
    fn rejects_windows_style_path_like_resource_file_names() {
        let game_dir = test_dir("reject-windows-path-resource");

        let error = apply_plan_resource_actions(
            &plan_with_removal(Some("folder\\outside.jar")),
            &empty_manifest(),
            &game_dir,
        )
        .expect_err("expected unsafe path error");

        assert!(error.contains("managed resource file name"));
    }

    #[test]
    fn skips_resource_removal_without_file_name() {
        let game_dir = test_dir("skip-missing-resource-file-name");

        let results =
            apply_plan_resource_actions(&plan_with_removal(None), &empty_manifest(), &game_dir)
                .expect("apply resource actions");

        assert_eq!(
            results,
            vec![ManagedResourceResult {
                resource_id: "old-mod".to_string(),
                action: ManagedResourceAction::SkippedNoFileName,
                path: None
            }]
        );
    }

    #[test]
    fn verifies_existing_direct_resource_with_matching_hash() {
        let game_dir = test_dir("verify-existing-direct-resource");
        let mods_dir = game_dir.join("mods");
        fs::create_dir_all(&mods_dir).expect("create mods dir");
        let file_path = mods_dir.join("direct.jar");
        fs::write(&file_path, "direct file").expect("write managed file");
        let resource = direct_resource(
            "direct-mod",
            "direct.jar",
            ManifestResourceHashes {
                sha512: None,
                sha256: Some(
                    "2c735545895a65a94dd6f3b3fc3624280771fa64a263d6ed182a602ee7c04d6c".to_string(),
                ),
            },
        );

        let results = apply_plan_resource_actions(
            &plan_with_sync("direct-mod", "direct.jar"),
            &manifest_with_resources(vec![resource]),
            &game_dir,
        )
        .expect("apply resource actions");

        assert_eq!(
            results,
            vec![ManagedResourceResult {
                resource_id: "direct-mod".to_string(),
                action: ManagedResourceAction::Verified,
                path: Some(file_path)
            }]
        );
    }

    #[test]
    fn downloads_and_verifies_a_direct_resource() {
        let body = b"downloaded direct file";
        let url = serve_once(body);
        let game_dir = test_dir("download-direct-resource");
        let hash = hex_digest(&Sha256::digest(body));
        let resource = direct_resource_at("direct-mod", "direct.jar", &url, hashes(&hash));

        let results = apply_plan_resource_actions(
            &plan_with_sync("direct-mod", "direct.jar"),
            &manifest_with_resources(vec![resource]),
            &game_dir,
        )
        .expect("apply resource actions");
        let installed_path = game_dir.join("mods/direct.jar");

        assert_eq!(
            fs::read(&installed_path).expect("read installed file"),
            body
        );
        assert_eq!(
            results,
            vec![ManagedResourceResult {
                resource_id: "direct-mod".to_string(),
                action: ManagedResourceAction::Downloaded,
                path: Some(installed_path),
            }]
        );
    }

    #[test]
    fn rejects_a_bad_download_without_replacing_the_existing_resource() {
        let url = serve_once(b"wrong file");
        let game_dir = test_dir("reject-bad-direct-resource");
        let mods_dir = game_dir.join("mods");
        fs::create_dir_all(&mods_dir).expect("create mods dir");
        let installed_path = mods_dir.join("direct.jar");
        fs::write(&installed_path, b"existing file").expect("write existing file");
        let expected_hash = hex_digest(&Sha256::digest(b"expected file"));
        let resource = direct_resource_at("direct-mod", "direct.jar", &url, hashes(&expected_hash));

        let error = apply_plan_resource_actions(
            &plan_with_sync("direct-mod", "direct.jar"),
            &manifest_with_resources(vec![resource]),
            &game_dir,
        )
        .expect_err("reject mismatched download");

        assert!(error.contains("did not match its expected hash"));
        assert_eq!(
            fs::read(installed_path).expect("read existing file"),
            b"existing file"
        );
    }

    fn plan_with_removal(file_name: Option<&str>) -> InstallPlan {
        plan_with_actions(vec![SetupActionPreview {
            id: "remove_resource_old-mod".to_string(),
            kind: SetupActionKind::RemoveResource,
            intent: SetupActionIntent::Remove,
            status: SetupActionStatus::Ready,
            required: false,
            resource_id: Some("old-mod".to_string()),
            subject: Some("Old Mod".to_string()),
            target: Some(SetupActionTarget::Mods),
            file_name: file_name.map(str::to_string),
        }])
    }

    fn plan_with_sync(resource_id: &str, file_name: &str) -> InstallPlan {
        plan_with_actions(vec![SetupActionPreview {
            id: format!("resource_{resource_id}"),
            kind: SetupActionKind::SyncResource,
            intent: SetupActionIntent::Add,
            status: SetupActionStatus::Ready,
            required: true,
            resource_id: Some(resource_id.to_string()),
            subject: Some(resource_id.to_string()),
            target: Some(SetupActionTarget::Mods),
            file_name: Some(file_name.to_string()),
        }])
    }

    fn plan_with_actions(actions: Vec<SetupActionPreview>) -> InstallPlan {
        InstallPlan {
            server_id: "example".to_string(),
            update_status: ServerUpdateStatus::UpdateAvailable,
            minecraft_version: "1.21.6".to_string(),
            fabric_loader_version: "0.16.14".to_string(),
            game_directory_name: "Example".to_string(),
            server_name: "Example".to_string(),
            server_address: "play.example.com".to_string(),
            launcher: LauncherKind::Official,
            profile: "balanced".to_string(),
            profile_label: "Balanced".to_string(),
            recommended_memory_mb: 4096,
            actions,
            required_mods: vec![],
            optional_mods: vec![],
            warnings: vec![],
        }
    }

    fn empty_manifest() -> SetupManifest {
        manifest_with_resources(vec![])
    }

    fn manifest_with_resources(resources: Vec<ManifestResource>) -> SetupManifest {
        SetupManifest {
            schema_version: 1,
            manifest_version: "1".to_string(),
            id: "example".to_string(),
            display_name: "Example".to_string(),
            server: ManifestServer {
                name: "Example".to_string(),
                address: "play.example.com".to_string(),
            },
            minecraft: ManifestMinecraft {
                version: "1.21.6".to_string(),
                loader: ManifestLoader {
                    kind: ManifestLoaderKind::None,
                    version: None,
                },
            },
            install: ManifestInstall {
                game_directory_name: "Example".to_string(),
                launcher_profile_name: "Example".to_string(),
            },
            profiles: vec![],
            resources,
            server_entry: None,
        }
    }

    fn direct_resource(
        id: &str,
        file_name: &str,
        hashes: ManifestResourceHashes,
    ) -> ManifestResource {
        direct_resource_at(
            id,
            file_name,
            &format!("https://example.com/{file_name}"),
            hashes,
        )
    }

    fn direct_resource_at(
        id: &str,
        file_name: &str,
        url: &str,
        hashes: ManifestResourceHashes,
    ) -> ManifestResource {
        ManifestResource {
            id: id.to_string(),
            name: id.to_string(),
            resource_type: ManifestResourceType::Mod,
            target: ManifestResourceTarget::Mods,
            required: true,
            profiles: vec![],
            file_name: Some(file_name.to_string()),
            source: ManifestResourceSource::Direct {
                url: url.to_string(),
            },
            hashes,
        }
    }

    fn hashes(sha256: &str) -> ManifestResourceHashes {
        ManifestResourceHashes {
            sha512: None,
            sha256: Some(sha256.to_string()),
        }
    }

    fn serve_once(body: &'static [u8]) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let address = listener.local_addr().expect("read test server address");

        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request).expect("read request");
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .expect("write response headers");
            stream.write_all(body).expect("write response body");
        });

        format!("http://{address}/resource.jar")
    }

    fn test_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_nanos();
        let path = env::temp_dir().join(format!("minecraft-setup-manager-{name}-{unique}"));
        fs::create_dir_all(&path).expect("create test dir");
        path
    }
}
