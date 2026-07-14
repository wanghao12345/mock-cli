use anyhow::Context;
use clap::Parser;
use colored::*;
use mock_cli_core::parse_spec;
use mock_cli_server::build_router;
use std::{
    fs,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::Arc,
};
use tokio::{
    net::TcpListener,
    signal,
};

#[derive(Parser, Debug)]
#[command(version, about = "🚀 Blazing fast OpenAPI Mock Server")]
struct Cli {
    /// Path to the OpenAPI spec file (YAML or JSON).
    spec: PathBuf,
    /// Port to listen on.
    #[arg(short, long, default_value = "3333")]
    port: u16,
    /// Host/IP address to bind to.
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    host: String,
}

/// Determine whether the spec content is JSON or YAML.
///
/// Strategy:
/// 1. Trust the file extension when it is explicit (`.json` / `.yaml` / `.yml`).
/// 2. Otherwise inspect the first non-whitespace byte: `{` or `[` → JSON, else YAML.
///    This makes the tool tolerant of files without a conventional extension
///    (e.g. `spec.txt` or stdin-saved content), instead of failing on extension alone.
fn detect_format(path: &PathBuf, content: &str) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => true,
        Some("yaml") | Some("yml") => false,
        _ => {
            // Fall back to content sniffing.
            let first = content.trim_start().chars().next().unwrap_or(' ');
            first == '{' || first == '['
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse CLI arguments.
    let cli = Cli::parse();

    let content = fs::read_to_string(&cli.spec).with_context(|| {
        format!(
            "{} Failed to read file: {}",
            "✗".red().bold(),
            cli.spec.display()
        )
    })?;

    // Detect format by extension first, then by content sniffing (#13).
    let is_json = detect_format(&cli.spec, &content);

    let spec = Arc::new(parse_spec(&content, is_json).with_context(|| {
        format!(
            "{} Failed to parse spec as {}: {}",
            "✗".red().bold(),
            if is_json { "JSON" } else { "YAML" },
            cli.spec.display()
        )
    })?);
    println!(
        "{} Loaded {:?} ({} paths)",
        "✓".green().bold(),
        cli.spec,
        spec.paths.paths.len()
    );

    // Parse and validate the bind host (#10).
    let host: IpAddr = cli.host.parse().with_context(|| {
        format!(
            "{} Invalid host '{}': expected an IP address like 127.0.0.1 or 0.0.0.0",
            "✗".red().bold(),
            cli.host
        )
    })?;
    let addr = SocketAddr::from((host, cli.port));

    let router = build_router(spec);
    // Bind BEFORE printing the Listening line so that a failed bind never
    // produces a misleading "Listening" message (#11).
    let listener = TcpListener::bind(&addr).await.with_context(|| {
        format!(
            "{} Failed to bind {}: port {} may be in use. Try another port with -p <PORT>",
            "✗".red().bold(),
            addr,
            cli.port
        )
    })?;
    println!("{} Listening on {}\n", "✓".green().bold(), &addr);

    // Graceful shutdown on Ctrl+C / SIGTERM (#12): in-flight requests finish,
    // then the server exits cleanly instead of being killed mid-response.
    let shutdown = async move {
        #[cfg(unix)]
        {
            // SIGTERM and Ctrl+C both trigger a clean shutdown.
            let sigterm = async {
                match signal::unix::signal(signal::unix::SignalKind::terminate()) {
                    Ok(mut s) => s.recv().await,
                    Err(_) => std::future::pending::<Option<()>>().await,
                }
            };
            tokio::select! {
                _ = signal::ctrl_c() => {}
                _ = sigterm => {}
            }
        }
        #[cfg(not(unix))]
        {
            let _ = signal::ctrl_c().await;
        }
        eprintln!("\n{} Shutting down...", "✓".green().bold());
    };

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown)
        .await?;
    Ok(())
}
