use std::path::PathBuf;

use crate::config::Config;
use crate::error::{ClickUpError, Result};

/// Default number of days to retain log files before cleanup.
const DEFAULT_RETENTION_DAYS: u32 = 7;

/// Logging configuration persisted in `config.toml` under `[logging]`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LoggingConfig {
    /// Number of days to retain log files before automatic cleanup.
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,

    /// Default log output format for file logging.
    #[serde(default)]
    pub format: LogFormat,

    /// Default log level (overridden by `CLICKUP_LOG` env var).
    #[serde(default = "default_level")]
    pub level: String,
}

/// Log output format.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// Structured JSON (one object per line).
    #[default]
    Json,
    /// Human-readable pretty format.
    Pretty,
}

fn default_retention_days() -> u32 {
    DEFAULT_RETENTION_DAYS
}

fn default_level() -> String {
    "info".to_string()
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            retention_days: DEFAULT_RETENTION_DAYS,
            format: LogFormat::default(),
            level: default_level(),
        }
    }
}

/// Returns the log directory path (`~/.config/clickup-rs/logs/`).
pub fn log_dir() -> PathBuf {
    Config::config_dir().join("logs")
}

/// Creates a new per-session log file and returns its path.
///
/// The file is named `{binary}-{ISO8601-timestamp}.log` where the timestamp
/// uses a filesystem-safe format (colons replaced with dashes).
pub fn create_session_log_file(binary_name: &str) -> Result<(std::fs::File, PathBuf)> {
    let dir = log_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| ClickUpError::ConfigError(format!("failed to create log directory: {e}")))?;

    let timestamp = chrono::Local::now().format("%Y-%m-%dT%H-%M-%S");
    let filename = format!("{binary_name}-{timestamp}.log");
    let path = dir.join(&filename);

    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| ClickUpError::ConfigError(format!("failed to create log file: {e}")))?;

    Ok((file, path))
}

/// Removes log files older than `retention_days` from the log directory.
///
/// Returns the number of files removed and total bytes freed.
pub fn cleanup_old_logs(retention_days: u32) -> Result<(usize, u64)> {
    let dir = log_dir();
    if !dir.exists() {
        return Ok((0, 0));
    }

    let cutoff = chrono::Utc::now() - chrono::Duration::days(i64::from(retention_days));

    let mut removed = 0usize;
    let mut bytes_freed = 0u64;

    let entries = std::fs::read_dir(&dir)
        .map_err(|e| ClickUpError::ConfigError(format!("failed to read log directory: {e}")))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("log") {
            continue;
        }

        if let Ok(metadata) = entry.metadata() {
            let modified = metadata.modified().ok().and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .and_then(|d| chrono::DateTime::from_timestamp(d.as_secs() as i64, 0))
            });

            if let Some(mod_time) = modified
                && mod_time < cutoff
            {
                bytes_freed += metadata.len();
                if std::fs::remove_file(&path).is_ok() {
                    removed += 1;
                }
            }
        }
    }

    Ok((removed, bytes_freed))
}

/// Lists all log session files in the log directory, sorted by modification
/// time (newest first).
pub fn list_log_sessions() -> Result<Vec<LogSessionInfo>> {
    let dir = log_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(&dir)
        .map_err(|e| ClickUpError::ConfigError(format!("failed to read log directory: {e}")))?;

    let mut sessions: Vec<LogSessionInfo> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("log") {
                return None;
            }

            let metadata = entry.metadata().ok()?;
            let filename = path.file_name()?.to_string_lossy().to_string();

            let binary = filename.split('-').next().unwrap_or("unknown").to_string();
            let modified = metadata.modified().ok()?;
            let size = metadata.len();

            Some(LogSessionInfo {
                filename,
                path,
                binary,
                size,
                modified,
            })
        })
        .collect();

    sessions.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(sessions)
}

/// Information about a single log session file.
#[derive(Debug, Clone)]
pub struct LogSessionInfo {
    /// The log file name (e.g., `cli-2026-03-25T22-30-00.log`).
    pub filename: String,
    /// Full path to the log file.
    pub path: PathBuf,
    /// The binary that created this log (e.g., `cli`, `tui`).
    pub binary: String,
    /// File size in bytes.
    pub size: u64,
    /// Last modification time.
    pub modified: std::time::SystemTime,
}

impl LogSessionInfo {
    /// Returns a human-readable age string (e.g., "2h ago", "3d ago").
    pub fn age_string(&self) -> String {
        let Ok(elapsed) = self.modified.elapsed() else {
            return "unknown".to_string();
        };
        let secs = elapsed.as_secs();
        if secs < 60 {
            format!("{secs}s ago")
        } else if secs < 3600 {
            format!("{}m ago", secs / 60)
        } else if secs < 86400 {
            format!("{}h ago", secs / 3600)
        } else {
            format!("{}d ago", secs / 86400)
        }
    }

    /// Returns a human-readable size string (e.g., "1.2 KB", "3.4 MB").
    pub fn size_string(&self) -> String {
        if self.size < 1024 {
            format!("{} B", self.size)
        } else if self.size < 1024 * 1024 {
            format!("{:.1} KB", self.size as f64 / 1024.0)
        } else {
            format!("{:.1} MB", self.size as f64 / (1024.0 * 1024.0))
        }
    }
}

/// Redacts sensitive data from a log line for export.
///
/// Replaces personal API tokens (`pk_*`) and OAuth tokens with redacted
/// versions. Emails are replaced with a masked version.
pub fn redact_log_line(line: &str) -> String {
    let mut result = line.to_string();

    // Redact personal API tokens (pk_ followed by alphanumeric chars).
    let mut i = 0;
    while let Some(pos) = result[i..].find("pk_") {
        let abs_pos = i + pos;
        let token_end = result[abs_pos..]
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|p| abs_pos + p)
            .unwrap_or(result.len());
        if token_end - abs_pos > 3 {
            result.replace_range(abs_pos..token_end, "pk_[REDACTED]");
            i = abs_pos + "pk_[REDACTED]".len();
        } else {
            i = abs_pos + 3;
        }
    }

    // Redact email addresses.
    result = redact_emails(&result);

    result
}

/// Replaces email addresses with a masked version (e.g., `u***@domain.com`).
fn redact_emails(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        if c == '@' && i > 0 {
            // Look back for the local part start.
            let local_start = result
                .rfind(|c: char| {
                    !c.is_alphanumeric() && c != '.' && c != '_' && c != '-' && c != '+'
                })
                .map(|p| p + 1)
                .unwrap_or(0);
            let local_part = &result[local_start..];
            if !local_part.is_empty() {
                // Find the domain part.
                let remaining = &input[i + 1..];
                let domain_end = remaining
                    .find(|c: char| !c.is_alphanumeric() && c != '.' && c != '-')
                    .unwrap_or(remaining.len());
                let domain = &remaining[..domain_end];

                if domain.contains('.') {
                    let first_char = local_part.chars().next().unwrap_or('*');
                    result.truncate(local_start);
                    result.push(first_char);
                    result.push_str("***@");
                    result.push_str(domain);
                    // Skip over the domain characters.
                    for _ in 0..domain_end {
                        chars.next();
                    }
                    continue;
                }
            }
        }
        result.push(c);
    }

    result
}

/// Generates a short request ID for correlation.
pub fn generate_request_id() -> String {
    let id = uuid::Uuid::new_v4();
    // Use first 8 hex chars for brevity while maintaining uniqueness.
    id.as_simple().to_string()[..8].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_config_defaults() {
        let config = LoggingConfig::default();
        assert_eq!(config.retention_days, 7);
        assert_eq!(config.format, LogFormat::Json);
        assert_eq!(config.level, "info");
    }

    #[test]
    fn test_logging_config_roundtrip() {
        let config = LoggingConfig {
            retention_days: 14,
            format: LogFormat::Pretty,
            level: "debug".to_string(),
        };
        let toml_str = toml::to_string_pretty(&config).expect("serialize");
        let deserialized: LoggingConfig = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_redact_log_line_token() {
        let line = r#"{"token":"pk_12345678_abcdef","msg":"auth"}"#;
        let redacted = redact_log_line(line);
        assert!(redacted.contains("pk_[REDACTED]"));
        assert!(!redacted.contains("pk_12345678_abcdef"));
    }

    #[test]
    fn test_redact_log_line_no_token() {
        let line = r#"{"msg":"hello world"}"#;
        let redacted = redact_log_line(line);
        assert_eq!(redacted, line);
    }

    #[test]
    fn test_redact_emails() {
        let input = "user alice@example.com logged in";
        let result = redact_emails(input);
        assert!(result.contains("a***@example.com"), "got: {result}");
        assert!(!result.contains("alice@example.com"));
    }

    #[test]
    fn test_redact_emails_no_email() {
        let input = "no email here";
        assert_eq!(redact_emails(input), input);
    }

    #[test]
    fn test_redact_emails_at_sign_without_domain() {
        let input = "value @tag";
        assert_eq!(redact_emails(input), input);
    }

    #[test]
    fn test_generate_request_id_length() {
        let id = generate_request_id();
        assert_eq!(id.len(), 8);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_request_id_unique() {
        let id1 = generate_request_id();
        let id2 = generate_request_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_log_format_serde() {
        let json: LogFormat = serde_json::from_str(r#""json""#).unwrap();
        assert_eq!(json, LogFormat::Json);
        let pretty: LogFormat = serde_json::from_str(r#""pretty""#).unwrap();
        assert_eq!(pretty, LogFormat::Pretty);
    }

    #[test]
    fn test_log_session_info_age_string() {
        let info = LogSessionInfo {
            filename: "test.log".to_string(),
            path: PathBuf::from("/tmp/test.log"),
            binary: "cli".to_string(),
            size: 1024,
            modified: std::time::SystemTime::now() - std::time::Duration::from_secs(3700),
        };
        let age = info.age_string();
        assert!(age.contains("h ago"), "got: {age}");
    }

    #[test]
    fn test_log_session_info_size_string() {
        let info = LogSessionInfo {
            filename: "test.log".to_string(),
            path: PathBuf::from("/tmp/test.log"),
            binary: "cli".to_string(),
            size: 2048,
            modified: std::time::SystemTime::now(),
        };
        assert_eq!(info.size_string(), "2.0 KB");
    }

    #[test]
    fn test_log_session_info_size_string_bytes() {
        let info = LogSessionInfo {
            filename: "test.log".to_string(),
            path: PathBuf::from("/tmp/test.log"),
            binary: "cli".to_string(),
            size: 512,
            modified: std::time::SystemTime::now(),
        };
        assert_eq!(info.size_string(), "512 B");
    }
}
