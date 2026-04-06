use std::io::{BufRead, Write};

use anyhow::{Context, Result};
use clap::Subcommand;

use clickup_api::logging;

use crate::output;

/// Log management subcommands.
#[derive(Subcommand)]
pub enum LogsCommands {
    /// List recent log sessions.
    List,

    /// Display a log file contents.
    Show {
        /// Session filename or index (from `logs list`). Defaults to the most
        /// recent session.
        session: Option<String>,

        /// Show only the last N lines.
        #[arg(long)]
        tail: Option<usize>,
    },

    /// Export logs for a bug report with sensitive data redacted.
    Export {
        /// Include sessions from the last N hours (default: 24).
        #[arg(long, default_value = "24")]
        hours: u64,

        /// Write to a file instead of stdout.
        #[arg(long, short)]
        output: Option<String>,
    },

    /// Remove old log files.
    Clean {
        /// Remove logs older than N days (default: 7).
        #[arg(long, default_value = "7")]
        older_than: u32,

        /// Show what would be removed without deleting.
        #[arg(long)]
        dry_run: bool,
    },
}

impl LogsCommands {
    /// Dispatches to the appropriate logs handler.
    pub async fn run(self) -> Result<()> {
        match self {
            Self::List => list_sessions(),
            Self::Show { session, tail } => show_session(session, tail),
            Self::Export { hours, output: out } => export_logs(hours, out),
            Self::Clean {
                older_than,
                dry_run,
            } => clean_logs(older_than, dry_run),
        }
    }
}

fn list_sessions() -> Result<()> {
    let sessions = logging::list_log_sessions().context("failed to list log sessions")?;

    if sessions.is_empty() {
        output::info("No log sessions found.");
        output::info(&format!("Log directory: {}", logging::log_dir().display()));
        return Ok(());
    }

    output::print_table(
        &["#", "Filename", "Binary", "Size", "Age"],
        sessions
            .iter()
            .enumerate()
            .map(|(i, s)| {
                vec![
                    format!("{}", i + 1),
                    s.filename.clone(),
                    s.binary.clone(),
                    s.size_string(),
                    s.age_string(),
                ]
            })
            .collect(),
    );

    output::info(&format!(
        "{} session(s) in {}",
        sessions.len(),
        logging::log_dir().display()
    ));

    Ok(())
}

fn show_session(session: Option<String>, tail: Option<usize>) -> Result<()> {
    let sessions = logging::list_log_sessions().context("failed to list log sessions")?;

    if sessions.is_empty() {
        output::info("No log sessions found.");
        return Ok(());
    }

    let target = match session {
        None => sessions[0].clone(),
        Some(ref s) => {
            // Try as index first.
            if let Ok(idx) = s.parse::<usize>() {
                if idx == 0 || idx > sessions.len() {
                    anyhow::bail!("index {idx} out of range (1..{})", sessions.len());
                }
                sessions[idx - 1].clone()
            } else {
                // Try as filename (exact or partial match).
                sessions
                    .iter()
                    .find(|sess| sess.filename == *s || sess.filename.contains(s.as_str()))
                    .cloned()
                    .context(format!("no session matching '{s}'"))?
            }
        }
    };

    output::info(&format!(
        "Session: {} ({}, {})",
        target.filename,
        target.size_string(),
        target.age_string()
    ));

    let file = std::fs::File::open(&target.path).context("failed to open log file")?;
    let reader = std::io::BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<std::io::Result<Vec<_>>>()?;

    let display_lines = match tail {
        Some(n) => &lines[lines.len().saturating_sub(n)..],
        None => &lines,
    };

    for line in display_lines {
        println!("{line}");
    }

    Ok(())
}

fn export_logs(hours: u64, output_path: Option<String>) -> Result<()> {
    let sessions = logging::list_log_sessions().context("failed to list log sessions")?;

    let cutoff = std::time::SystemTime::now() - std::time::Duration::from_secs(hours * 3600);

    let recent: Vec<_> = sessions.iter().filter(|s| s.modified >= cutoff).collect();

    if recent.is_empty() {
        output::info(&format!("No log sessions in the last {hours}h."));
        return Ok(());
    }

    let mut buf = String::new();

    // Header.
    buf.push_str("=== clickup-rs log export ===\n");
    buf.push_str(&format!(
        "Generated: {}\n",
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")
    ));
    buf.push_str(&format!(
        "OS: {} {}\n",
        std::env::consts::OS,
        std::env::consts::ARCH
    ));
    buf.push_str("Version: clickup-rs 0.1.0\n");
    buf.push_str(&format!(
        "Config: {}\n",
        clickup_api::config::Config::config_file_path().display()
    ));
    buf.push_str(&format!("Sessions: {} (last {hours}h)\n", recent.len()));
    buf.push_str("===\n\n");

    // Each session.
    for session in &recent {
        buf.push_str(&format!("[session: {}]\n", session.filename));

        match std::fs::read_to_string(&session.path) {
            Ok(content) => {
                for line in content.lines() {
                    buf.push_str(&logging::redact_log_line(line));
                    buf.push('\n');
                }
            }
            Err(e) => {
                buf.push_str(&format!("(error reading file: {e})\n"));
            }
        }
        buf.push('\n');
    }

    match output_path {
        Some(path) => {
            let mut file = std::fs::File::create(&path)
                .context(format!("failed to create output file: {path}"))?;
            file.write_all(buf.as_bytes())?;
            output::success(&format!("Exported {} session(s) to {path}", recent.len()));
        }
        None => {
            print!("{buf}");
        }
    }

    Ok(())
}

fn clean_logs(older_than: u32, dry_run: bool) -> Result<()> {
    if dry_run {
        let sessions = logging::list_log_sessions().context("failed to list log sessions")?;

        let cutoff = std::time::SystemTime::now()
            - std::time::Duration::from_secs(u64::from(older_than) * 86400);

        let old: Vec<_> = sessions.iter().filter(|s| s.modified < cutoff).collect();

        if old.is_empty() {
            output::info(&format!("No log files older than {older_than} day(s)."));
        } else {
            output::info(&format!("Would remove {} file(s):", old.len()));
            for s in &old {
                println!("  {} ({})", s.filename, s.size_string());
            }
        }
    } else {
        let (removed, bytes) =
            logging::cleanup_old_logs(older_than).context("failed to clean logs")?;

        if removed == 0 {
            output::info(&format!("No log files older than {older_than} day(s)."));
        } else {
            let freed = if bytes < 1024 {
                format!("{bytes} B")
            } else if bytes < 1024 * 1024 {
                format!("{:.1} KB", bytes as f64 / 1024.0)
            } else {
                format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
            };
            output::success(&format!("Removed {removed} log file(s), freed {freed}"));
        }
    }

    Ok(())
}
