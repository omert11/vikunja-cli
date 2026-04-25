use anyhow::Result;
use clap::{Parser, Subcommand};

mod client;
mod commands;
mod config;
mod output;
mod types;
mod util;

use commands::{label, project, task};

#[derive(Parser)]
#[command(name = "vikunja-cli")]
#[command(version, about = "CLI for Vikunja task management", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output as JSON
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Task operations
    Task {
        #[command(subcommand)]
        cmd: task::TaskCmd,
    },
    /// Project operations
    Project {
        #[command(subcommand)]
        cmd: project::ProjectCmd,
    },
    /// Label operations
    Label {
        #[command(subcommand)]
        cmd: label::LabelCmd,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = config::load()?;
    let client = client::VikunjaClient::new(&cfg)?;

    match cli.command {
        Commands::Task { cmd } => task::run(cmd, &client, cli.json).await,
        Commands::Project { cmd } => project::run(cmd, &client, cli.json).await,
        Commands::Label { cmd } => label::run(cmd, &client, cli.json).await,
    }
}
