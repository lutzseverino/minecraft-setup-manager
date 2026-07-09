use sha2::{Digest, Sha256};

use super::schema::SetupManifest;

pub fn manifest_fingerprint(manifest: &SetupManifest) -> Result<String, String> {
    let canonical = serde_json::to_vec(manifest)
        .map_err(|error| format!("Could not fingerprint the setup manifest: {error}"))?;
    let digest = Sha256::digest(canonical);

    Ok(format!("sha256:{}", to_hex(&digest)))
}

fn to_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }

    output
}
