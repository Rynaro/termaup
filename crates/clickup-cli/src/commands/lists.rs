use anyhow::{Context, Result};
use clap::Subcommand;

use crate::client_factory::create_client;
use crate::output;

/// List subcommands.
#[derive(Subcommand)]
pub enum ListCommands {
    /// List all lists in a space or folder.
    List {
        /// Space ID — lists all lists (folders + folderless).
        #[arg(long, group = "source")]
        space: Option<String>,

        /// Folder ID — lists only lists in this folder.
        #[arg(long, group = "source")]
        folder: Option<String>,
    },
    /// Show details for a list.
    Get {
        /// The list ID.
        list_id: String,
    },
}

impl ListCommands {
    /// Dispatches to the appropriate list handler.
    pub async fn run(self, format: &str, workspace_override: Option<&str>) -> Result<()> {
        match self {
            Self::List { space, folder } => {
                list_lists(format, workspace_override, space, folder).await
            }
            Self::Get { list_id } => get_list(format, workspace_override, &list_id).await,
        }
    }
}

async fn list_lists(
    format: &str,
    workspace_override: Option<&str>,
    space: Option<String>,
    folder: Option<String>,
) -> Result<()> {
    let (client, _config) = create_client(workspace_override)?;

    // Collect (list, folder_name) pairs.
    let mut items: Vec<(clickup_api::models::List, String)> = Vec::new();

    if let Some(folder_id) = folder {
        let lists = client
            .get_lists_in_folder(&folder_id)
            .await
            .context("failed to fetch lists in folder")?;
        for l in lists {
            let fname = l.folder.name.clone().unwrap_or_else(|| folder_id.clone());
            items.push((l, fname));
        }
    } else if let Some(space_id) = space {
        // Fetch folders and their lists.
        let folders = client
            .get_folders(&space_id)
            .await
            .context("failed to fetch folders")?;
        for f in &folders {
            let lists = client
                .get_lists_in_folder(&f.id)
                .await
                .context("failed to fetch lists in folder")?;
            for l in lists {
                items.push((l, f.name.clone()));
            }
        }

        // Fetch folderless lists.
        let folderless = client
            .get_folderless_lists(&space_id)
            .await
            .context("failed to fetch folderless lists")?;
        for l in folderless {
            items.push((l, "—".to_string()));
        }
    } else {
        anyhow::bail!("provide --space or --folder to specify which lists to show");
    }

    if format == "json" {
        let lists: Vec<&clickup_api::models::List> = items.iter().map(|(l, _)| l).collect();
        output::print_json(&lists);
        return Ok(());
    }

    if items.is_empty() {
        output::info("No lists found.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = items
        .iter()
        .map(|(l, folder_name)| {
            vec![
                l.id.clone(),
                l.name.clone(),
                folder_name.clone(),
                l.task_count.clone().unwrap_or_else(|| "—".to_string()),
            ]
        })
        .collect();

    output::print_table(&["ID", "Name", "Folder", "Tasks"], rows);

    Ok(())
}

async fn get_list(format: &str, workspace_override: Option<&str>, list_id: &str) -> Result<()> {
    let (client, _config) = create_client(workspace_override)?;

    let list = client
        .get_list(list_id)
        .await
        .context("failed to fetch list")?;

    if format == "json" {
        output::print_json(&list);
        return Ok(());
    }

    output::info(&format!("List: {} ({})", list.name, list.id));
    println!(
        "  Task count: {}",
        list.task_count.as_deref().unwrap_or("—")
    );
    if let Some(content) = &list.content
        && !content.is_empty()
    {
        println!("  Description: {content}");
    }

    Ok(())
}
