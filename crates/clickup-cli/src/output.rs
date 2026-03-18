use comfy_table::{Cell, CellAlignment, Color as TableColor, Table};
use owo_colors::OwoColorize;
use serde::Serialize;

/// Prints a success message with a green "✓" prefix.
pub fn success(msg: &str) {
    println!("{} {msg}", "✓".green());
}

/// Prints an error message with a red "✗" prefix to stderr.
pub fn error(msg: &str) {
    eprintln!("{} {msg}", "✗".red());
}

/// Prints an informational message with a blue "ℹ" prefix.
pub fn info(msg: &str) {
    println!("{} {msg}", "ℹ".blue());
}

/// Prints a bordered table with coloured headers.
pub fn print_table(headers: &[&str], rows: Vec<Vec<String>>) {
    let mut table = Table::new();
    table.set_content_arrangement(comfy_table::ContentArrangement::DynamicFullWidth);

    let header_cells: Vec<Cell> = headers
        .iter()
        .map(|h| {
            Cell::new(h)
                .set_alignment(CellAlignment::Left)
                .fg(TableColor::Cyan)
        })
        .collect();
    table.set_header(header_cells);

    for row in rows {
        table.add_row(row);
    }

    println!("{table}");
}

/// Pretty-prints a serialisable value as JSON.
pub fn print_json<T: Serialize>(data: &T) {
    match serde_json::to_string_pretty(data) {
        Ok(json) => println!("{json}"),
        Err(e) => error(&format!("failed to serialize JSON: {e}")),
    }
}

/// Renders a markdown string to the terminal using `termimad`.
pub fn print_markdown(md: &str) {
    let skin = termimad::MadSkin::default();
    skin.print_text(md);
}

/// Applies a hex colour to a status label for terminal display.
///
/// Falls back to plain text when the colour cannot be parsed.
pub fn format_status(status: &str, color: &str) -> String {
    if let Some((r, g, b)) = parse_hex_color(color) {
        format!("{}", status.truecolor(r, g, b))
    } else {
        status.to_string()
    }
}

/// Maps a priority label to a coloured emoji string.
pub fn format_priority(priority: Option<&str>) -> String {
    match priority {
        Some("urgent") => "🔴 Urgent".to_string(),
        Some("high") => "🟠 High".to_string(),
        Some("normal") => "🟡 Normal".to_string(),
        Some("low") => "🔵 Low".to_string(),
        _ => "⚪ None".to_string(),
    }
}

/// Parses a ClickUp millisecond timestamp into a human-readable date.
pub fn format_date(timestamp: Option<&str>) -> String {
    let Some(ts) = timestamp else {
        return "—".to_string();
    };
    let Ok(ms) = ts.parse::<i64>() else {
        return ts.to_string();
    };

    chrono::DateTime::from_timestamp_millis(ms)
        .map(|dt| dt.format("%b %d, %Y").to_string())
        .unwrap_or_else(|| ts.to_string())
}

/// Parses a hex colour string (e.g. `#ff00aa` or `ff00aa`) into RGB.
fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}
