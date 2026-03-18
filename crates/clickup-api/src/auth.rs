use std::path::PathBuf;

use crate::config::Config;
use crate::error::{ClickUpError, Result};

const SERVICE_NAME: &str = "clickup-rs";
const USERNAME: &str = "api_token";

/// Manages API token storage via the OS keyring with a file-based fallback.
pub struct TokenStorage;

impl TokenStorage {
    /// Stores the API token in the OS keyring and the file fallback.
    ///
    /// Both backends are written so that the token is available across
    /// different processes and container instances sharing the config volume.
    pub fn store_token(token: &str) -> Result<()> {
        // Always persist to file so the token survives across containers and
        // processes that may not share the same keyring session.
        Self::store_token_file(token)?;

        // Best-effort store in the OS keyring for native desktop use.
        if let Ok(entry) = keyring::Entry::new(SERVICE_NAME, USERNAME) {
            let _ = entry.set_password(token);
        }

        Ok(())
    }

    /// Retrieves the API token.
    ///
    /// Resolution order: `CLICKUP_TOKEN` env var → OS keyring → file fallback.
    pub fn get_token() -> Result<String> {
        // 1. Environment variable (useful in CI and Docker).
        if let Ok(token) = std::env::var("CLICKUP_TOKEN")
            && !token.is_empty()
        {
            return Ok(token);
        }

        // 2. OS keyring.
        if let Ok(entry) = keyring::Entry::new(SERVICE_NAME, USERNAME)
            && let Ok(token) = entry.get_password()
        {
            return Ok(token);
        }

        // 3. File fallback.
        Self::get_token_file()
    }

    /// Deletes the API token from the OS keyring, falling back to a file.
    pub fn delete_token() -> Result<()> {
        match keyring::Entry::new(SERVICE_NAME, USERNAME) {
            Ok(entry) => match entry.delete_credential() {
                Ok(()) => {
                    // Also remove fallback file if it exists.
                    let path = Self::token_file_path();
                    if path.exists() {
                        let _ = std::fs::remove_file(&path);
                    }
                    Ok(())
                }
                Err(_) => Self::delete_token_file(),
            },
            Err(_) => Self::delete_token_file(),
        }
    }

    fn token_file_path() -> PathBuf {
        Config::config_dir().join(".token")
    }

    fn store_token_file(token: &str) -> Result<()> {
        let dir = Config::config_dir();
        std::fs::create_dir_all(&dir).map_err(|e| {
            ClickUpError::ConfigError(format!("Failed to create config directory: {e}"))
        })?;

        let path = Self::token_file_path();
        std::fs::write(&path, token)
            .map_err(|e| ClickUpError::ConfigError(format!("Failed to write token file: {e}")))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&path, perms).map_err(|e| {
                ClickUpError::ConfigError(format!("Failed to set token file permissions: {e}"))
            })?;
        }

        Ok(())
    }

    fn get_token_file() -> Result<String> {
        let path = Self::token_file_path();
        std::fs::read_to_string(&path)
            .map_err(|e| ClickUpError::ConfigError(format!("Failed to read token file: {e}")))
    }

    fn delete_token_file() -> Result<()> {
        let path = Self::token_file_path();
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| {
                ClickUpError::ConfigError(format!("Failed to delete token file: {e}"))
            })?;
        }
        Ok(())
    }
}
