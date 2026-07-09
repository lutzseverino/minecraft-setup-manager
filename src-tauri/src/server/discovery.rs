use reqwest::Url;

use super::address::parse_server_address;

const MANIFEST_PATH: &str = "/.well-known/minecraft-setup-manager/manifest.json";

pub fn manifest_url_for_address(input: &str) -> Result<(String, String), String> {
    let trimmed = input.trim();

    if has_http_scheme(trimmed) {
        return direct_manifest_url(trimmed);
    }

    let address = parse_server_address(trimmed)?;

    Ok((
        address.normalized().to_string(),
        format!("https://{}{MANIFEST_PATH}", address.discovery_host()),
    ))
}

fn has_http_scheme(input: &str) -> bool {
    ["http://", "https://"].into_iter().any(|scheme| {
        input
            .get(..scheme.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(scheme))
    })
}

fn direct_manifest_url(input: &str) -> Result<(String, String), String> {
    let parsed = Url::parse(input).map_err(|_| "The setup file URL is not valid.".to_string())?;

    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.fragment().is_some()
    {
        return Err(
            "Use a valid HTTP or HTTPS setup file URL without a password or fragment.".to_string(),
        );
    }

    let normalized = parsed.to_string();
    Ok((normalized.clone(), normalized))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_manifests_without_reusing_the_minecraft_port() {
        assert_eq!(
            manifest_url_for_address("play.example.com:25566").unwrap(),
            (
                "play.example.com:25566".to_string(),
                "https://play.example.com/.well-known/minecraft-setup-manager/manifest.json"
                    .to_string(),
            )
        );
    }

    #[test]
    fn discovers_manifests_for_ipv6_servers() {
        assert_eq!(
            manifest_url_for_address("[2001:db8::1]:25565").unwrap(),
            (
                "[2001:db8::1]:25565".to_string(),
                "https://[2001:db8::1]/.well-known/minecraft-setup-manager/manifest.json"
                    .to_string(),
            )
        );
    }

    #[test]
    fn keeps_valid_direct_manifest_urls() {
        assert_eq!(
            manifest_url_for_address("https://config.example.com/setup.json?channel=stable")
                .unwrap(),
            (
                "https://config.example.com/setup.json?channel=stable".to_string(),
                "https://config.example.com/setup.json?channel=stable".to_string(),
            )
        );
        assert!(manifest_url_for_address("HTTPS://config.example.com/setup.json").is_ok());
    }

    #[test]
    fn rejects_credentials_and_fragments_in_direct_urls() {
        assert!(manifest_url_for_address("https://user@example.com/setup.json").is_err());
        assert!(manifest_url_for_address("https://example.com/setup.json#old").is_err());
    }
}
