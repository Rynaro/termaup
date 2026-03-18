mod client_factory;
mod commands;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use commands::auth::AuthCommands;
use commands::lists::ListCommands;
use commands::spaces::SpaceCommands;
use commands::tasks::TaskCommands;
use commands::workspaces::WorkspaceCommands;

/// A fast CLI client for ClickUp, built in Rust.
#[derive(Parser)]
#[command(name = "clickup", about, version)]
struct Cli {
    /// Output format.
    #[arg(long, default_value = "table", global = true)]
    format: String,

    /// Override the default workspace ID.
    #[arg(long, global = true)]
    workspace: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authentication management.
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
    /// Manage workspaces.
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommands,
    },
    /// Browse and view spaces.
    Space {
        #[command(subcommand)]
        command: SpaceCommands,
    },
    /// Browse and view lists.
    List {
        #[command(subcommand)]
        command: ListCommands,
    },
    /// Browse and view tasks.
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("CLICKUP_LOG").unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    let fmt = cli.format.as_str();
    let ws = cli.workspace.as_deref();

    let result = match cli.command {
        Commands::Auth { command } => command.run().await,
        Commands::Workspace { command } => command.run(fmt, ws).await,
        Commands::Space { command } => command.run(fmt, ws).await,
        Commands::List { command } => command.run(fmt, ws).await,
        Commands::Task { command } => command.run(fmt, ws).await,
    };

    if let Err(e) = result {
        output::error(&format!("{e:#}"));
        std::process::exit(1);
    }

    Ok(())
}
