use std::fs;
use std::io::{Read, Write};
use std::net::IpAddr;
use std::path::Path;
use std::time::Duration;

use reqwest::blocking::{Client, Response};
use reqwest::redirect::{Action, Attempt, Policy};
use reqwest::Url;

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
) -> Result<(), String> {
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
    })
}

fn get(url: &str, description: &str) -> Result<Response, String> {
    validate_url(url)?;
    let client = Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .redirect(Policy::custom(redirect_policy))
        .user_agent(concat!("MinecraftSetupManager/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| format!("Could not prepare the download client: {error}"))?;

    client
        .get(url)
        .send()
        .map_err(|error| format!("Could not reach the {description}: {error}"))?
        .error_for_status()
        .map_err(|error| format!("The {description} could not be downloaded: {error}"))
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

    if is_allowed_url(attempt.url()) {
        attempt.follow()
    } else {
        attempt.error("redirected to an insecure URL")
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

fn is_allowed_url(url: &Url) -> bool {
    match url.scheme() {
        "https" => true,
        "http" => url
            .host_str()
            .is_some_and(|host| host.eq_ignore_ascii_case("localhost") || is_loopback_ip(host)),
        _ => false,
    }
}

fn is_loopback_ip(host: &str) -> bool {
    host.trim_start_matches('[')
        .trim_end_matches(']')
        .parse::<IpAddr>()
        .is_ok_and(|ip| ip.is_loopback())
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
    }
}
