use std::{fs, path::PathBuf};
use anyhow::Context;
use clap::Parser;
use mock_cli_core::{generate_mock, parse_spec};



#[derive(Parser, Debug)]
#[command(version, about = "🚀 Blazing fast OpenAPI Mock Server")]
struct Cli {
    /// Path to the OpenAPI spec file.
    spec: PathBuf,
    /// Port to listen on.
    #[arg(short, long, default_value = "3333")]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse the command line arguments.
    let cli = Cli::parse();
    let content = fs::read_to_string(&cli.spec).with_context(|| format!("Failed to read file: {}", cli.spec.display()))?;
    // Check if the file is a JSON file.
    let is_json = cli.spec.extension().map_or(false, |ext| ext == "json");

    // Parse the spec.
    let spec = parse_spec(&content, is_json)?;
    // Generate the mock server.
    generate_mock(&spec)?;



    Ok(())
}
