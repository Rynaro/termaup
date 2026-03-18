use anyhow::{Context, Result};
use clap::Subcommand;
use dialoguer::Select;

use clickup_api::auth::TokenStorage;
use clickup_api::client::ClickUpClient;
use clickup_api::config::Config;

use crate::output;

/// Authentication subcommands.
#[derive(Subcommand)]
pub enum AuthCommands {
    /// Log in with a ClickUp API token.
    Login {
        /// Personal API token (starts with pk_). Prompted interactively if
        /// omitted.
        #[arg(long)]
        token: Option<String>,

        /// Workspace ID to set as default (skips interactive selection).
        #[arg(long)]
        workspace: Option<String>,
    },

    /// Show current authentication status.
    Status,

    /// Log out and remove stored credentials.
    Logout,

    /// Switch the default workspace.
    Switch {
        /// Workspace ID to switch to (skips interactive selection).
        #[arg(long)]
        workspace: Option<String>,
    },
}

impl AuthCommands {
    /// Dispatches to the appropriate auth handler.
    pub async fn run(self) -> Result<()> {
        match self {
            Self::Login { token, workspace } => login(token, workspace).await,
            Self::Status => status().await,
            Self::Logout => logout(),
            Self::Switch { workspace } => switch(workspace).await,
        }
    }
}

/// Creates a [`ClickUpClient`] from a token, using the configured base URL.
fn make_client(token: &str) -> Result<ClickUpClient> {
    let config = Config::load().context("failed to load configuration")?;
    Ok(ClickUpClient::with_base_url(
        token.to_string(),
        config.api_base_url,
    ))
}

async fn login(token_flag: Option<String>, workspace_flag: Option<String>) -> Result<()> {
    let token = match token_flag {
        Some(t) => t,
        None => {
            let t = rpassword::prompt_password("Enter your ClickUp API token: ")
                .context("failed to read token")?;
            if t.trim().is_empty() {
                anyhow::bail!("token cannot be empty");
            }
            t.trim().to_string()
        }
    };

    // Validate token.
    let client = make_client(&token)?;
    let user = client
        .get_authenticated_user()
        .await
        .context("token validation failed")?;

    // Persist token.
    TokenStorage::store_token(&token).context("failed to store token")?;

    output::success(&format!(
        "Authenticated as {} ({})",
        user.username, user.email
    ));

    // Fetch workspaces.
    let workspaces = client
        .get_workspaces()
        .await
        .context("failed to fetch workspaces")?;

    if workspaces.is_empty() {
        output::info("No workspaces found.");
        return Ok(());
    }

    output::info("Available workspaces:");
    for (i, ws) in workspaces.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, ws.name, ws.id);
    }

    // Determine default workspace.
    let mut config = Config::load().context("failed to load configuration")?;

    let selected_id = if let Some(id) = workspace_flag {
        // Validate the provided ID exists.
        if !workspaces.iter().any(|w| w.id == id) {
            anyhow::bail!("workspace ID '{id}' not found in your workspaces");
        }
        id
    } else if workspaces.len() == 1 {
        workspaces[0].id.clone()
    } else {
        let items: Vec<String> = workspaces
            .iter()
            .map(|w| format!("{} ({})", w.name, w.id))
            .collect();

        let selection = Select::new()
            .with_prompt("Select a default workspace")
            .items(&items)
            .default(0)
            .interact()
            .context("workspace selection cancelled")?;

        workspaces[selection].id.clone()
    };

    let ws_name = workspaces
        .iter()
        .find(|w| w.id == selected_id)
        .map(|w| w.name.as_str())
        .unwrap_or("unknown");

    config.default_workspace_id = Some(selected_id);
    config.save().context("failed to save configuration")?;

    output::success(&format!("Default workspace set to: {ws_name}"));

    Ok(())
}

async fn status() -> Result<()> {
    let token = match TokenStorage::get_token() {
        Ok(t) => t,
        Err(_) => {
            output::error("Not authenticated. Run `clickup auth login`");
            return Ok(());
        }
    };

    let client = make_client(&token)?;

    match client.get_authenticated_user().await {
        Ok(user) => {
            output::success(&format!(
                "Authenticated as {} ({})",
                user.username, user.email
            ));

            let config = Config::load().context("failed to load configuration")?;
            match &config.default_workspace_id {
                Some(id) => {
                    output::info(&format!("Default workspace: {id}"));
                }
                None => {
                    output::info(
                        "No default workspace set. \
                         Run `clickup auth switch` to select one.",
                    );
                }
            }
        }
        Err(e) => {
            output::error(&format!("Token is invalid or expired: {e}"));
            output::info("Run `clickup auth login` to re-authenticate.");
        }
    }

    Ok(())
}

fn logout() -> Result<()> {
    TokenStorage::delete_token().context("failed to delete token")?;
    output::success("Logged out successfully");
    Ok(())
}

async fn switch(workspace_flag: Option<String>) -> Result<()> {
    let token =
        TokenStorage::get_token().context("not authenticated — run `clickup auth login` first")?;

    let client = make_client(&token)?;
    let workspaces = client
        .get_workspaces()
        .await
        .context("failed to fetch workspaces")?;

    if workspaces.is_empty() {
        output::info("No workspaces found.");
        return Ok(());
    }

    let selected_id = if let Some(id) = workspace_flag {
        if !workspaces.iter().any(|w| w.id == id) {
            anyhow::bail!("workspace ID '{id}' not found in your workspaces");
        }
        id
    } else {
        let items: Vec<String> = workspaces
            .iter()
            .map(|w| format!("{} ({})", w.name, w.id))
            .collect();

        let selection = Select::new()
            .with_prompt("Select a workspace")
            .items(&items)
            .default(0)
            .interact()
            .context("workspace selection cancelled")?;

        workspaces[selection].id.clone()
    };

    let ws_name = workspaces
        .iter()
        .find(|w| w.id == selected_id)
        .map(|w| w.name.as_str())
        .unwrap_or("unknown");

    let mut config = Config::load().context("failed to load configuration")?;
    config.default_workspace_id = Some(selected_id);
    config.save().context("failed to save configuration")?;

    output::success(&format!("Switched to workspace: {ws_name}"));

    Ok(())
}
