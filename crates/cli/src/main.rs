use std::path::PathBuf;
use clap::Parser;



#[derive(Parser, Debug)]
struct Cli {
    /// Path to the OpenAPI spec file.
    spec: PathBuf,
    /// Port to listen on.
    port: u16,
}


fn main() {
    println!("Hello, world!");
}
