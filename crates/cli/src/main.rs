use anyhow::Context;
use clap::Parser;
use mock_cli_core::parse_spec;
use mock_cli_server::build_router;
use std::{fs, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::net::TcpListener;

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
    let content = fs::read_to_string(&cli.spec)
        .with_context(|| format!("Failed to read file: {}", cli.spec.display()))?;
    // Check if the file is a JSON file.
    let is_json = cli.spec.extension().map_or(false, |ext| ext == "json");

    let spec = Arc::new(parse_spec(&content, is_json)?);
    println!("✓ Loaded {:?} ({} paths)", cli.spec, spec.paths.paths.len());

    let addr = SocketAddr::from(([127, 0, 0, 1], cli.port));
    println!("✓ Listening on {}\n", &addr);

    let router = build_router(spec);
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
