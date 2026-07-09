use super::schema::SetupManifest;
use crate::http_client;

const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;

pub fn fetch_manifest(manifest_url: &str) -> Result<SetupManifest, String> {
    let bytes = http_client::get_bytes(manifest_url, MAX_MANIFEST_BYTES, "setup file")?;

    let manifest = serde_json::from_slice(&bytes)
        .map_err(|error| format!("The server's setup file is not valid: {error}"))?;
    super::validation::validate_manifest(&manifest, manifest_url)?;
    Ok(manifest)
}
