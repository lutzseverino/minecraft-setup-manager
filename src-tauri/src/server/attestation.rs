use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::http_client;

const ATTESTATION_PATH: &str = "/.well-known/minecraft-setup-manager/attestations";
const MAX_RESPONSE_BYTES: u64 = 16 * 1024;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AttestationWireRequest<'a> {
    protocol_version: u8,
    challenge: &'a str,
    manifest_fingerprint: &'a str,
    profile_id: &'a str,
    client: AttestationClient<'a>,
}

#[derive(Serialize)]
struct AttestationClient<'a> {
    name: &'a str,
    version: &'a str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttestationWireResponse {
    status: String,
    manifest_fingerprint: String,
}

pub fn redeem(
    manifest_url: &str,
    challenge: &str,
    manifest_fingerprint: &str,
    profile_id: &str,
) -> Result<(), String> {
    let normalized_challenge = normalize_challenge(challenge)?;
    let endpoint = attestation_url_for_manifest(manifest_url)?;
    let request = AttestationWireRequest {
        protocol_version: 1,
        challenge: &normalized_challenge,
        manifest_fingerprint,
        profile_id,
        client: AttestationClient {
            name: "minecraft-setup-manager",
            version: env!("CARGO_PKG_VERSION"),
        },
    };
    let response: AttestationWireResponse = http_client::post_json(
        endpoint.as_str(),
        &request,
        MAX_RESPONSE_BYTES,
        "server setup check",
    )?;

    if response.status != "accepted" || response.manifest_fingerprint != manifest_fingerprint {
        return Err("The server did not confirm the setup check.".to_string());
    }
    Ok(())
}

fn attestation_url_for_manifest(manifest_url: &str) -> Result<Url, String> {
    let mut url = Url::parse(manifest_url)
        .map_err(|_| "The saved setup file URL is not valid.".to_string())?;
    url.set_path(ATTESTATION_PATH);
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

fn normalize_challenge(input: &str) -> Result<String, String> {
    let normalized = input
        .chars()
        .filter(|character| *character != '-')
        .flat_map(char::to_uppercase)
        .collect::<String>();
    let valid = normalized.len() == 16
        && normalized.chars().all(|character| {
            matches!(character, '0'..='9' | 'A'..='H' | 'J' | 'K' | 'M' | 'N' | 'P'..='T' | 'V'..='Z')
        });
    if !valid {
        return Err("Enter the 16-character setup code shown by Minecraft.".to_string());
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_attestation_from_the_exact_manifest_origin() {
        assert_eq!(
            attestation_url_for_manifest("https://config.example.com/custom/setup.json?stable=1")
                .unwrap()
                .as_str(),
            "https://config.example.com/.well-known/minecraft-setup-manager/attestations"
        );
    }

    #[test]
    fn normalizes_human_readable_codes() {
        assert_eq!(
            normalize_challenge("0123-4567-89ab-cdef").unwrap(),
            "0123456789ABCDEF"
        );
        assert!(normalize_challenge("0123-ILOU-89AB-CDEF").is_err());
    }
}
