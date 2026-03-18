use anyhow::{Context, Result};
use clap::Subcommand;

use crate::client_factory::create_client;
use crate::output;

/// Workspace subcommands.
#[derive(Subcommand)]
pub enum WorkspaceCommands {
    /// List all workspaces.
    List,
}

impl WorkspaceCommands {
    /// Dispatches to the appropriate workspace handler.
    pub async fn run(self, format: &str, workspace_override: Option<&str>) -> Result<()> {
        match self {
            Self::List => list_workspaces(format, workspace_override).await,
        }
    }
}

async fn list_workspaces(format: &str, workspace_override: Option<&str>) -> Result<()> {
    let (client, _config) = create_client(workspace_override)?;
    let workspaces = client
        .get_workspaces()
        .await
        .context("failed to fetch workspaces")?;

    if format == "json" {
        output::print_json(&workspaces);
        return Ok(());
    }

    if workspaces.is_empty() {
        output::info("No workspaces found.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = workspaces
        .iter()
        .map(|w| vec![w.id.clone(), w.name.clone(), w.members.len().to_string()])
        .collect();

    output::print_table(&["ID", "Name", "Members"], rows);

    Ok(())
}
