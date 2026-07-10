use sha2::{Digest, Sha256};

use super::schema::SetupManifest;

pub fn manifest_fingerprint(manifest: &SetupManifest) -> Result<String, String> {
    let mut normalized = manifest.clone();
    for resource in &mut normalized.resources {
        resource.profiles.sort();
        resource.hashes.sha512 = resource
            .hashes
            .sha512
            .as_ref()
            .map(|hash| hash.to_lowercase());
        resource.hashes.sha256 = resource
            .hashes
            .sha256
            .as_ref()
            .map(|hash| hash.to_lowercase());
    }
    let canonical = serde_jcs::to_vec(&normalized)
        .map_err(|error| format!("Could not fingerprint the setup manifest: {error}"))?;
    let digest = Sha256::digest(canonical);

    Ok(format!("msm-v1-sha256:{}", to_hex(&digest)))
}

fn to_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_protocol_golden_fingerprint() {
        let bytes = include_bytes!("../../../protocol/fixtures/v1/valid/minimal-vanilla.json");
        let manifest = crate::manifest::parse::parse_manifest(
            bytes,
            "https://setup.example.com/manifest.json",
        )
        .expect("parse protocol fixture");

        assert_eq!(
            manifest_fingerprint(&manifest).expect("fingerprint manifest"),
            "msm-v1-sha256:1dfce0075ec03e4bdc2bfe58b6006af3c738814b3de33f053bed5017628df0a3"
        );
    }
}
