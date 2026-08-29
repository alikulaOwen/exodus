//! Project Exodus CLI binary entry point.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "exodus",
    author,
    version,
    about = "Graph-guided, agent-assisted legacy code migration engine"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Parse legacy repository source files
    Parse {
        #[arg(short, long)]
        path: PathBuf,
    },
    /// Build and inspect the Exodus Semantic Graph
    Graph {
        #[arg(short, long)]
        path: PathBuf,
    },
    /// Generate an ordered migration plan
    Plan {
        #[arg(short, long)]
        path: PathBuf,
    },
    /// Execute transformation and migration loop
    Migrate {
        #[arg(short, long)]
        plan: PathBuf,
    },
    /// Verify generated target codebase
    Verify {
        #[arg(short, long)]
        target: PathBuf,
    },
    /// Output evaluation and outcome metrics
    Report {
        #[arg(short, long)]
        evidence: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { path } => {
            println!("Parsing legacy source at: {}", path.display());
        }
        Commands::Graph { path } => {
            println!("Constructing semantic graph for: {}", path.display());
        }
        Commands::Plan { path } => {
            println!("Generating migration plan for: {}", path.display());
        }
        Commands::Migrate { plan } => {
            println!("Executing migration plan from: {}", plan.display());
        }
        Commands::Verify { target } => {
            println!("Verifying target at: {}", target.display());
        }
        Commands::Report { evidence } => {
            println!("Generating report from evidence at: {}", evidence.display());
        }
    }

    Ok(())
}
