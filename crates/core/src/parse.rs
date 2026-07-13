use anyhow::{Context, Result};
use openapiv3::OpenAPI;

/// Parse the OpenAPI spec content.
pub fn parse_spec(content: &str, is_json: bool) -> Result<OpenAPI> {
    if is_json {
        serde_json::from_str(content).context("Failed to parse JSON spec")
    } else {
        serde_yaml::from_str(content).context("Failed to parse YAML spec")
    }
}
