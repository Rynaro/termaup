use serde::{Deserialize, Serialize};

use crate::serde_helpers::{deserialize_i32_or_string, deserialize_option_string_or_number};

/// Status of a task or list in ClickUp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    /// Status ID (may be absent on some responses).
    #[serde(default, deserialize_with = "deserialize_option_string_or_number")]
    pub id: Option<String>,
    /// Status name (e.g. "Open", "In Progress").
    #[serde(default)]
    pub status: String,
    /// Hex colour for the status.
    #[serde(default)]
    pub color: String,
    /// Status type (e.g. "open", "closed", "custom").
    #[serde(default, rename = "type")]
    pub status_type: String,
    /// Display order index.
    #[serde(default, deserialize_with = "deserialize_i32_or_string")]
    pub orderindex: i32,
}
