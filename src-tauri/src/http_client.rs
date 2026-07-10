use std::fs;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::path::Path;
use std::time::Duration;

use reqwest::blocking::{Client, Response};
use reqwest::redirect::{Action, Attempt, Policy};
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::Serialize;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5 * 60);

pub fn get_bytes(url: &str, max_bytes: u64, description: &str) -> Result<Vec<u8>, String> {
    let response = get(url, description)?;
    read_limited(response, max_bytes, description)
}

pub fn download_to_path(
    url: &str,
    path: &Path,
    max_bytes: u64,
    description: &str,
) -> Result<u64, String> {
    let response = get(url, description)?;
    let mut reader = response.take(max_bytes + 1);
    let mut file = fs::File::create(path).map_err(|error| {
        format!(
            "Could not create the temporary {description} at {}: {error}",
            path.display()
        )
    })?;
    let copied = std::io::copy(&mut reader, &mut file).map_err(|error| {
        let _ = fs::remove_file(path);
        format!("Could not save the {description}: {error}")
    })?;

    if copied > max_bytes {
        drop(file);
        let _ = fs::remove_file(path);
        return Err(format!("The {description} is too large."));
    }

    file.flush().map_err(|error| {
        let _ = fs::remove_file(path);
        format!("Could not finish saving the {description}: {error}")
    })?;

    Ok(copied)
}

pub fn post_json<Request: Serialize, ResponseBody: DeserializeOwned>(
    url: &str,
    body: &Request,
    max_response_bytes: u64,
    description: &str,
) -> Result<ResponseBody, String> {
    let (url, addresses) = prepare_url(url)?;
    let response = client(false, &url, &addresses)?
        .post(url)
        .json(body)
        .send()
        .map_err(|error| format!("Could not reach the {description}: {error}"))?;
    let status = response.status();
    let bytes = read_limited(response, max_response_bytes, description)?;

    if !status.is_success() {
        return Err(protocol_error_message(status.as_u16(), &bytes, description));
    }

    serde_json::from_slice(&bytes)
        .map_err(|error| format!("The {description} returned an invalid response: {error}"))
}

fn get(url: &str, description: &str) -> Result<Response, String> {
    let (url, addresses) = prepare_url(url)?;
    client(true, &url, &addresses)?
        .get(url)
        .send()
        .map_err(|error| format!("Could not reach the {description}: {error}"))?
        .error_for_status()
        .map_err(|error| format!("The {description} could not be downloaded: {error}"))
}

fn client(
    follow_same_origin_redirects: bool,
    url: &Url,
    addresses: &[SocketAddr],
) -> Result<Client, String> {
    let redirects = if follow_same_origin_redirects {
        Policy::custom(redirect_policy)
    } else {
        Policy::none()
    };
    let host = url
        .host_str()
        .ok_or_else(|| "The network URL has no host.".to_string())?;
    Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .redirect(redirects)
        .user_agent(concat!("MinecraftSetupManager/", env!("CARGO_PKG_VERSION")))
        .resolve_to_addrs(host, addresses)
        .build()
        .map_err(|error| format!("Could not prepare the network client: {error}"))
}

fn read_limited(response: Response, max_bytes: u64, description: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    response
        .take(max_bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Could not read the {description}: {error}"))?;

    if bytes.len() as u64 > max_bytes {
        return Err(format!("The {description} is too large."));
    }

    Ok(bytes)
}

fn redirect_policy(attempt: Attempt<'_>) -> Action {
    if attempt.previous().len() >= 5 {
        return attempt.error("too many redirects");
    }

    let same_origin = attempt
        .previous()
        .last()
        .is_some_and(|previous| has_same_origin(previous, attempt.url()));
    if same_origin && is_allowed_url(attempt.url()) {
        attempt.follow()
    } else {
        attempt.error("redirected outside the secure download origin")
    }
}

fn validate_url(url: &str) -> Result<(), String> {
    let parsed = Url::parse(url).map_err(|_| "The download URL is not valid.".to_string())?;

    if is_allowed_url(&parsed) {
        Ok(())
    } else {
        Err("Downloads must use HTTPS. HTTP is only allowed for this computer.".to_string())
    }
}

fn prepare_url(value: &str) -> Result<(Url, Vec<SocketAddr>), String> {
    validate_url(value)?;
    let url = Url::parse(value).map_err(|_| "The network URL is not valid.".to_string())?;
    let host = url
        .host_str()
        .ok_or_else(|| "The network URL has no host.".to_string())?;
    let port = url
        .port_or_known_default()
        .ok_or_else(|| "The network URL has no usable port.".to_string())?;
    let addresses = (host, port)
        .to_socket_addrs()
        .map_err(|_| "The server address could not be found.".to_string())?
        .collect::<Vec<_>>();
    if addresses.is_empty() {
        return Err("The server address did not resolve to a network address.".to_string());
    }

    let loopback_target = is_loopback_host(host);
    let unsafe_address = addresses.iter().any(|address| {
        if loopback_target {
            !address.ip().is_loopback()
        } else {
            crate::manifest::validation::is_non_public_ip(address.ip())
        }
    });
    if unsafe_address {
        return Err("The server address resolved to a private or local network.".to_string());
    }

    Ok((url, addresses))
}

fn is_allowed_url(url: &Url) -> bool {
    match url.scheme() {
        "https" => url
            .host_str()
            .is_some_and(|host| is_loopback_host(host) || !is_explicitly_non_public_host(host)),
        "http" => url.host_str().is_some_and(is_loopback_host),
        _ => false,
    }
}

fn has_same_origin(left: &Url, right: &Url) -> bool {
    left.scheme() == right.scheme()
        && left.host_str() == right.host_str()
        && left.port_or_known_default() == right.port_or_known_default()
}

fn is_loopback_host(host: &str) -> bool {
    let host = host.trim_start_matches('[').trim_end_matches(']');
    host.eq_ignore_ascii_case("localhost")
        || host.ends_with(".localhost")
        || host.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())
}

fn is_explicitly_non_public_host(host: &str) -> bool {
    let host = host.trim_start_matches('[').trim_end_matches(']');
    host.ends_with(".local")
        || host
            .parse::<IpAddr>()
            .is_ok_and(crate::manifest::validation::is_non_public_ip)
}

fn protocol_error_message(status: u16, bytes: &[u8], description: &str) -> String {
    #[derive(serde::Deserialize)]
    struct Problem {
        code: Option<String>,
    }

    let code = serde_json::from_slice::<Problem>(bytes)
        .ok()
        .and_then(|problem| problem.code);
    match code.as_deref() {
        Some("challenge_invalid") => {
            "That setup code is not valid. Check the code shown by Minecraft.".to_string()
        }
        Some("challenge_expired") => {
            "That setup code expired. Try joining the server again to get a new code.".to_string()
        }
        Some("fingerprint_mismatch") => {
            "The server setup changed. Check the server again and apply the new setup.".to_string()
        }
        Some("profile_invalid") => {
            "The chosen setup option is no longer available. Check the server again.".to_string()
        }
        Some("rate_limited") => {
            "Too many setup codes were tried. Wait a minute and try again.".to_string()
        }
        Some("attestation_unavailable") => {
            "The server could not save this setup check. Try again soon.".to_string()
        }
        _ => format!("The {description} returned HTTP {status}."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_https_and_loopback_http() {
        assert!(validate_url("https://example.com/file.jar").is_ok());
        assert!(validate_url("http://localhost:8080/manifest.json").is_ok());
        assert!(validate_url("http://127.0.0.1:8080/file.jar").is_ok());
        assert!(validate_url("http://[::1]:8080/file.jar").is_ok());
    }

    #[test]
    fn rejects_remote_http_and_non_http_schemes() {
        assert!(validate_url("http://example.com/file.jar").is_err());
        assert!(validate_url("file:///tmp/file.jar").is_err());
        assert!(validate_url("https://192.168.1.10/file.jar").is_err());
        assert!(validate_url("https://server.local/file.jar").is_err());
    }

    #[test]
    fn redirects_must_keep_the_same_secure_origin() {
        let source = Url::parse("https://example.com/file.jar").unwrap();
        let same_origin = Url::parse("https://example.com/next.jar").unwrap();
        let other_origin = Url::parse("https://cdn.example.com/file.jar").unwrap();

        assert!(has_same_origin(&source, &same_origin));
        assert!(!has_same_origin(&source, &other_origin));
    }

    #[test]
    fn resolves_loopback_once_for_pinned_local_development() {
        let (url, addresses) = prepare_url("http://localhost:8765/manifest.json").unwrap();

        assert_eq!(url.host_str(), Some("localhost"));
        assert!(!addresses.is_empty());
        assert!(addresses.iter().all(|address| address.ip().is_loopback()));
    }
}
