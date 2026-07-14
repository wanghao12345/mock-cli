use anyhow::{Context, Result};
use openapiv3::OpenAPI;

/// Parse OpenAPI spec content.
///
/// `is_json` selects the deserializer: `true` -> JSON, `false` -> YAML.
/// The caller is expected to pick based on file extension or content sniffing
/// (see `cli::detect_format`).
pub fn parse_spec(content: &str, is_json: bool) -> Result<OpenAPI> {
    if is_json {
        serde_json::from_str(content).context("Failed to parse JSON spec")
    } else {
        serde_yaml::from_str(content).context("Failed to parse YAML spec")
    }
}
