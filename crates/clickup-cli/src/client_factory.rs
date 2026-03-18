use anyhow::Context;

use clickup_api::auth::TokenStorage;
use clickup_api::client::ClickUpClient;
use clickup_api::config::Config;

/// Loads the stored token and configuration, returning a ready-to-use client.
///
/// If `workspace_override` is `Some`, it replaces the configured default
/// workspace ID for this invocation (the config file is not modified).
pub fn create_client(workspace_override: Option<&str>) -> anyhow::Result<(ClickUpClient, Config)> {
    let token =
        TokenStorage::get_token().context("not authenticated — run `clickup auth login` first")?;

    let mut config = Config::load().context("failed to load configuration")?;

    if let Some(ws) = workspace_override {
        config.default_workspace_id = Some(ws.to_string());
    }

    let client = ClickUpClient::with_base_url(token, config.api_base_url.clone());

    Ok((client, config))
}
