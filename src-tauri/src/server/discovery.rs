use super::address::normalize_server_address;

pub fn manifest_url_for_address(input: &str) -> Result<(String, String), String> {
    let normalized = normalize_server_address(input)?;

    if normalized.starts_with("http://") || normalized.starts_with("https://") {
        return Ok((normalized.clone(), normalized));
    }

    let host = normalized
        .split(':')
        .next()
        .map(str::to_string)
        .ok_or_else(|| "Enter a server address.".to_string())?;

    Ok((
        normalized,
        format!("https://{host}/.well-known/minecraft-setup-manager/manifest.json"),
    ))
}
