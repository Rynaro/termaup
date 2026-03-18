use anyhow::{Context, Result};
use clap::Subcommand;

use crate::client_factory::create_client;
use crate::output;

/// Space subcommands.
#[derive(Subcommand)]
pub enum SpaceCommands {
    /// List spaces in a workspace.
    List {
        /// Workspace ID (overrides the default).
        #[arg(long)]
        workspace: Option<String>,
    },
    /// Show details for a space.
    Get {
        /// The space ID.
        space_id: String,
    },
}

impl SpaceCommands {
    /// Dispatches to the appropriate space handler.
    pub async fn run(self, format: &str, workspace_override: Option<&str>) -> Result<()> {
        match self {
            Self::List { workspace } => {
                let ws = workspace.as_deref().or(workspace_override);
                list_spaces(format, ws).await
            }
            Self::Get { space_id } => get_space(format, workspace_override, &space_id).await,
        }
    }
}

async fn list_spaces(format: &str, workspace_override: Option<&str>) -> Result<()> {
    let (client, config) = create_client(workspace_override)?;

    let team_id = config.default_workspace_id.as_deref().context(
        "no workspace specified — use --workspace or set a default \
             with `clickup auth switch`",
    )?;

    let spaces = client
        .get_spaces(team_id)
        .await
        .context("failed to fetch spaces")?;

    if format == "json" {
        output::print_json(&spaces);
        return Ok(());
    }

    if spaces.is_empty() {
        output::info("No spaces found.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = spaces
        .iter()
        .map(|s| {
            vec![
                s.id.clone(),
                s.name.clone(),
                if s.private { "Yes" } else { "No" }.to_string(),
                s.statuses.len().to_string(),
            ]
        })
        .collect();

    output::print_table(&["ID", "Name", "Private", "Statuses"], rows);

    Ok(())
}

async fn get_space(format: &str, workspace_override: Option<&str>, space_id: &str) -> Result<()> {
    let (client, _config) = create_client(workspace_override)?;

    let space = client
        .get_space(space_id)
        .await
        .context("failed to fetch space")?;

    if format == "json" {
        output::print_json(&space);
        return Ok(());
    }

    output::info(&format!("Space: {} ({})", space.name, space.id));
    println!("  Private: {}", if space.private { "Yes" } else { "No" });
    println!(
        "  Multiple assignees: {}",
        if space.multiple_assignees {
            "Yes"
        } else {
            "No"
        }
    );

    if !space.statuses.is_empty() {
        println!("\n  Statuses:");
        for s in &space.statuses {
            println!(
                "    {} ({})",
                output::format_status(&s.status, &s.color),
                s.status_type,
            );
        }
    }

    Ok(())
}
