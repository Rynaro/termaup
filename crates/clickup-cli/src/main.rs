mod client_factory;
mod commands;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use commands::auth::AuthCommands;
use commands::lists::ListCommands;
use commands::logs::LogsCommands;
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
    /// View, export, and manage log files.
    Logs {
        #[command(subcommand)]
        command: LogsCommands,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let env_filter =
        EnvFilter::try_from_env("CLICKUP_LOG").unwrap_or_else(|_| EnvFilter::new("warn"));

    let log_format = std::env::var("CLICKUP_LOG_FORMAT").unwrap_or_default();

    // Create per-session log file (best-effort).
    let log_file = clickup_api::logging::create_session_log_file("cli")
        .ok()
        .map(|(file, _path)| std::sync::Mutex::new(file));

    if log_format == "json" {
        // JSON on stderr.
        let json_stderr = tracing_subscriber::fmt::layer()
            .json()
            .with_writer(std::io::stderr);

        if let Some(file) = log_file {
            let json_file = tracing_subscriber::fmt::layer()
                .json()
                .with_writer(file)
                .with_ansi(false);
            tracing_subscriber::registry()
                .with(env_filter)
                .with(json_stderr)
                .with(json_file)
                .init();
        } else {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(json_stderr)
                .init();
        }
    } else {
        // Pretty on stderr.
        let stderr_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stderr);

        if let Some(file) = log_file {
            let json_file = tracing_subscriber::fmt::layer()
                .json()
                .with_writer(file)
                .with_ansi(false);
            tracing_subscriber::registry()
                .with(env_filter)
                .with(stderr_layer)
                .with(json_file)
                .init();
        } else {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(stderr_layer)
                .init();
        }
    }

    // Best-effort cleanup of old log files on startup.
    let retention = clickup_api::config::Config::load()
        .map(|c| c.logging.retention_days)
        .unwrap_or(7);
    let _ = clickup_api::logging::cleanup_old_logs(retention);

    let cli = Cli::parse();
    let fmt = cli.format.as_str();
    let ws = cli.workspace.as_deref();

    let result = match cli.command {
        Commands::Auth { command } => command.run().await,
        Commands::Workspace { command } => command.run(fmt, ws).await,
        Commands::Space { command } => command.run(fmt, ws).await,
        Commands::List { command } => command.run(fmt, ws).await,
        Commands::Task { command } => command.run(fmt, ws).await,
        Commands::Logs { command } => command.run().await,
    };

    if let Err(e) = result {
        output::error(&format!("{e:#}"));
        std::process::exit(1);
    }

    Ok(())
}
