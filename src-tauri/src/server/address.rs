use std::net::IpAddr;

use reqwest::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerAddress {
    normalized: String,
    discovery_host: String,
}

impl ServerAddress {
    pub fn normalized(&self) -> &str {
        &self.normalized
    }

    pub fn discovery_host(&self) -> &str {
        &self.discovery_host
    }
}

pub fn parse_server_address(input: &str) -> Result<ServerAddress, String> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err("Enter a server address.".to_string());
    }

    let without_scheme = trimmed
        .get(.."minecraft://".len())
        .filter(|prefix| prefix.eq_ignore_ascii_case("minecraft://"))
        .map_or(trimmed, |_| &trimmed["minecraft://".len()..])
        .trim_end_matches('/');

    if let Ok(ip) = without_scheme.parse::<IpAddr>() {
        return Ok(ip_address(ip, None));
    }

    let parsed = Url::parse(&format!("minecraft://{without_scheme}"))
        .map_err(|_| invalid_server_address())?;

    if !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || !matches!(parsed.path(), "" | "/")
    {
        return Err(invalid_server_address());
    }

    let host = parsed.host_str().ok_or_else(invalid_server_address)?;
    let port = parsed.port();

    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(ip_address(ip, port));
    }

    let normalized_host = host.to_ascii_lowercase();
    let normalized = port.map_or_else(
        || normalized_host.clone(),
        |port| format!("{normalized_host}:{port}"),
    );

    Ok(ServerAddress {
        normalized,
        discovery_host: normalized_host,
    })
}

fn ip_address(ip: IpAddr, port: Option<u16>) -> ServerAddress {
    let discovery_host = match ip {
        IpAddr::V4(address) => address.to_string(),
        IpAddr::V6(address) => format!("[{address}]"),
    };
    let normalized = port.map_or_else(
        || discovery_host.clone(),
        |port| format!("{discovery_host}:{port}"),
    );

    ServerAddress {
        normalized,
        discovery_host,
    }
}

fn invalid_server_address() -> String {
    "Use a server address like play.example.com or play.example.com:25565.".to_string()
}

pub fn server_key(normalized_address: &str, manifest_id: &str) -> String {
    format!("{manifest_id}@{normalized_address}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_hostnames_and_keeps_the_minecraft_port() {
        assert_eq!(
            parse_server_address(" MINECRAFT://Play.Example.com:25566/ ").unwrap(),
            ServerAddress {
                normalized: "play.example.com:25566".to_string(),
                discovery_host: "play.example.com".to_string(),
            }
        );
    }

    #[test]
    fn formats_ipv4_and_ipv6_hosts_for_https_discovery() {
        assert_eq!(
            parse_server_address("127.0.0.1:25565").unwrap(),
            ServerAddress {
                normalized: "127.0.0.1:25565".to_string(),
                discovery_host: "127.0.0.1".to_string(),
            }
        );
        assert_eq!(
            parse_server_address("[2001:db8::1]:25565").unwrap(),
            ServerAddress {
                normalized: "[2001:db8::1]:25565".to_string(),
                discovery_host: "[2001:db8::1]".to_string(),
            }
        );
        assert_eq!(
            parse_server_address("2001:db8::1").unwrap(),
            ServerAddress {
                normalized: "[2001:db8::1]".to_string(),
                discovery_host: "[2001:db8::1]".to_string(),
            }
        );
    }

    #[test]
    fn rejects_paths_queries_credentials_and_invalid_ports() {
        for address in [
            "play.example.com/path",
            "play.example.com?setup=1",
            "user@play.example.com",
            "play.example.com:99999",
        ] {
            assert!(parse_server_address(address).is_err(), "{address}");
        }
    }
}
