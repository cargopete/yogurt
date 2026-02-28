//! yogurt CLI — Rust toolchain for The Graph subgraphs

mod commands;
mod graph_node;
mod ipfs;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "yogurt")]
#[command(author, version, about = "Rust toolchain for The Graph subgraphs", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialise a new subgraph project
    Init {
        /// Project name
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Generate Rust types from schema and ABIs
    Codegen {
        /// Path to subgraph.yaml
        #[arg(short, long, default_value = "subgraph.yaml")]
        manifest: String,
    },

    /// Compile the subgraph to WASM
    Build {
        /// Release mode (optimised)
        #[arg(short, long)]
        release: bool,
    },

    /// Run mapping handler tests
    Test {
        /// Run tests in WASM (slower, higher fidelity)
        #[arg(long)]
        wasm: bool,
    },

    /// Deploy the subgraph to a local graph-node
    Deploy {
        /// Subgraph name (format: account/subgraph-name)
        name: Option<String>,

        /// Graph-node admin URL (default: http://localhost:8020)
        #[arg(long)]
        node: Option<String>,

        /// IPFS API URL (default: http://localhost:5001)
        #[arg(long)]
        ipfs: Option<String>,

        /// Version label for this deployment
        #[arg(short, long)]
        version: Option<String>,
    },

    /// Validate WASM exports for graph-node compatibility
    Validate {
        /// Path to compiled WASM file
        #[arg(default_value = "build/subgraph.wasm")]
        wasm_file: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => commands::init::run(name).await,
        Commands::Codegen { manifest } => commands::codegen::run(&manifest),
        Commands::Build { release } => commands::build::run(release),
        Commands::Test { wasm } => commands::test::run(wasm),
        Commands::Deploy { name, node, ipfs, version } => {
            commands::deploy::run(node, ipfs, name, version).await
        }
        Commands::Validate { wasm_file } => commands::validate::run(&wasm_file),
    }
}
