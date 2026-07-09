use super::schema::SetupManifest;

const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;

pub fn fetch_manifest(manifest_url: &str) -> Result<SetupManifest, String> {
    let response = reqwest::blocking::get(manifest_url)
        .map_err(|error| format!("Could not reach the setup manifest: {error}"))?;
    let status = response.status();

    if !status.is_success() {
        return Err(format!(
            "The setup manifest returned HTTP status {}.",
            status.as_u16()
        ));
    }

    let bytes = response
        .bytes()
        .map_err(|error| format!("Could not read the setup manifest: {error}"))?;

    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err("The setup manifest is too large.".to_string());
    }

    serde_json::from_slice(&bytes)
        .map_err(|error| format!("Could not parse the setup manifest: {error}"))
}
