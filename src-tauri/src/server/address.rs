pub fn normalize_server_address(input: &str) -> Result<String, String> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err("Enter a server address.".to_string());
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return Ok(trimmed.trim_end_matches('/').to_string());
    }

    let without_scheme = trimmed
        .trim_start_matches("minecraft://")
        .trim_end_matches('/');

    if without_scheme.contains('/') {
        return Err(
            "Use a server address like play.example.com, without extra path text.".to_string(),
        );
    }

    Ok(without_scheme.to_lowercase())
}

pub fn server_key(normalized_address: &str, manifest_id: &str) -> String {
    format!("{manifest_id}@{normalized_address}")
}
