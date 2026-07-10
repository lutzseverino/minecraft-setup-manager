use serde_json::Value;

use super::schema::SetupManifest;

pub fn parse_manifest(bytes: &[u8], source_url: &str) -> Result<SetupManifest, String> {
    let manifest: SetupManifest = serde_json::from_slice(bytes)
        .map_err(|error| format!("The server's setup file is not valid: {error}"))?;
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|error| format!("The server's setup file is not valid: {error}"))?;

    if contains_null(&value) {
        return Err(
            "The server's setup file must omit empty optional values instead of using null."
                .to_string(),
        );
    }

    super::validation::validate_manifest(&manifest, source_url)?;
    Ok(manifest)
}

fn contains_null(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::Array(values) => values.iter().any(contains_null),
        Value::Object(values) => values.values().any(contains_null),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use serde::Deserialize;

    use super::*;

    #[derive(Deserialize)]
    struct FixtureCatalog {
        valid: Vec<ValidFixture>,
        invalid: Vec<InvalidFixture>,
    }

    #[derive(Deserialize)]
    struct ValidFixture {
        path: String,
        fingerprint: String,
    }

    #[derive(Deserialize)]
    struct InvalidFixture {
        path: String,
    }

    #[test]
    fn rejects_explicit_nulls() {
        let bytes = include_bytes!("../../../protocol/fixtures/v1/invalid/explicit-null.json");
        let error = parse_manifest(bytes, "https://setup.example.com/manifest.json")
            .expect_err("explicit null must fail");

        assert!(error.contains("instead of using null"));
    }

    #[test]
    fn conforms_to_the_pinned_protocol_fixtures() {
        let fixture_root = protocol_fixture_root();
        let catalog: FixtureCatalog = serde_json::from_slice(
            &fs::read(fixture_root.join("catalog.json")).expect("read fixture catalog"),
        )
        .expect("parse fixture catalog");

        for fixture in catalog.valid {
            let bytes = fs::read(fixture_root.join(&fixture.path)).expect("read valid fixture");
            let manifest = parse_manifest(&bytes, "https://setup.example.com/manifest.json")
                .unwrap_or_else(|error| panic!("{} should be valid: {error}", fixture.path));
            let fingerprint = crate::manifest::fingerprint::manifest_fingerprint(&manifest)
                .expect("fingerprint valid fixture");
            assert_eq!(fingerprint, fixture.fingerprint, "{}", fixture.path);
        }

        for fixture in catalog.invalid {
            let bytes = fs::read(fixture_root.join(&fixture.path)).expect("read invalid fixture");
            assert!(
                parse_manifest(&bytes, "https://setup.example.com/manifest.json").is_err(),
                "{} should be invalid",
                fixture.path
            );
        }
    }

    fn protocol_fixture_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../protocol/fixtures/v1")
    }
}
