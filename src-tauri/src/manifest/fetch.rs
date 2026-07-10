use super::schema::SetupManifest;
use crate::http_client;

const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;

pub fn fetch_manifest(manifest_url: &str) -> Result<SetupManifest, String> {
    let bytes = http_client::get_bytes(manifest_url, MAX_MANIFEST_BYTES, "setup file")?;

    super::parse::parse_manifest(&bytes, manifest_url)
}
