use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::theme::{THEME, hex_to_color};
use crate::widgets::markdown;

/// Renders the task detail screen.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Task Detail ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(THEME.border_style());

    let Some(task) = &app.current_task else {
        let p = Paragraph::new("  Loading task…")
            .block(block)
            .style(THEME.muted_style());
        frame.render_widget(p, area);
        return;
    };

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line<'static>> = Vec::new();

    // ─── Header ───
    let status_color = hex_to_color(&task.status.color);

    let priority_str = task
        .priority
        .as_ref()
        .and_then(|p| p.priority.as_deref())
        .map(priority_emoji)
        .unwrap_or_else(|| "⚪".to_string());

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            task.name.clone(),
            Style::default().fg(THEME.fg).add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            format!(" {} ", task.status.status),
            Style::default()
                .fg(Color::Black)
                .bg(status_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(priority_str, Style::default()),
    ]));
    lines.push(Line::from(""));

    // ─── Metadata ───
    let assignees: String = if task.assignees.is_empty() {
        "—".to_string()
    } else {
        task.assignees
            .iter()
            .map(|u| u.username.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };

    let due = task
        .due_date
        .as_deref()
        .and_then(|s| format_timestamp(Some(s)))
        .unwrap_or_else(|| "—".to_string());

    let created =
        format_timestamp(Some(&task.date_created)).unwrap_or_else(|| task.date_created.clone());

    lines.push(meta_line("Assignees", &assignees));
    lines.push(meta_line("Due Date", &due));
    lines.push(meta_line("Created", &created));

    // Time tracking
    if let Some(estimate) = task.time_estimate {
        lines.push(meta_line("Estimate", &format_duration(estimate)));
    }
    if let Some(spent) = task.time_spent {
        lines.push(meta_line("Tracked", &format_duration(spent)));
    }

    if !task.tags.is_empty() {
        let tags: String = task
            .tags
            .iter()
            .map(|t| format!("#{}", t.name))
            .collect::<Vec<_>>()
            .join("  ");
        lines.push(meta_line("Tags", &tags));
    }

    lines.push(meta_line("URL", &task.url));
    lines.push(Line::from(""));

    // ─── Description ───
    if let Some(md) = &task.markdown_description {
        if !md.is_empty() {
            lines.push(section_header("Description"));
            lines.push(Line::from(""));

            let md_lines = markdown::render_markdown(md);
            for line in md_lines {
                let mut indented: Vec<Span<'static>> = vec![Span::raw("  ")];
                indented.extend(line.spans.into_iter());
                lines.push(Line::from(indented));
            }
        }
    } else if let Some(text) = &task.text_content
        && !text.is_empty()
    {
        lines.push(section_header("Description"));
        lines.push(Line::from(""));
        for text_line in text.lines() {
            lines.push(Line::from(format!("  {text_line}")));
        }
    }

    // ─── Subtasks ───
    if let Some(subtasks) = &task.subtasks
        && !subtasks.is_empty()
    {
        lines.push(Line::from(""));
        lines.push(section_header(&format!("Subtasks ({})", subtasks.len())));
        lines.push(Line::from(""));

        for st in subtasks {
            let st_color = hex_to_color(&st.status.color);

            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled("● ", Style::default().fg(st_color)),
                Span::raw(st.name.clone()),
                Span::styled(
                    format!("  [{}]", st.status.status),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    }

    // ─── Custom Fields ───
    if let Some(fields) = &task.custom_fields {
        let non_empty: Vec<_> = fields.iter().filter(|f| f.value.is_some()).collect();
        if !non_empty.is_empty() {
            lines.push(Line::from(""));
            lines.push(section_header(&format!(
                "Custom Fields ({})",
                non_empty.len()
            )));
            lines.push(Line::from(""));

            for field in &non_empty {
                let display = format_custom_field_value(&field.field_type, field.value.as_ref());
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("    {}: ", field.name),
                        Style::default()
                            .fg(THEME.muted)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(display, Style::default().fg(THEME.fg)),
                ]));
            }
        }
    }

    // ─── Checklists ───
    if !task.checklists.is_empty() {
        lines.push(Line::from(""));
        lines.push(section_header(&format!(
            "Checklists ({})",
            task.checklists.len()
        )));
        lines.push(Line::from(""));

        for cl in &task.checklists {
            let total = cl.items.len();
            let resolved = cl.items.iter().filter(|i| i.resolved).count();

            // Checklist name with progress
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(
                    cl.name.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  {resolved}/{total}"),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));

            // Progress bar
            if total > 0 {
                let bar_width = 20usize;
                let filled = (resolved as f64 / total as f64 * bar_width as f64).round() as usize;
                let empty = bar_width - filled;
                let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
                let bar_color = if resolved == total {
                    Color::Green
                } else {
                    Color::Yellow
                };
                lines.push(Line::from(vec![
                    Span::raw("    ["),
                    Span::styled(bar, Style::default().fg(bar_color)),
                    Span::raw("]"),
                ]));
            }

            // Items
            for item in &cl.items {
                let check = if item.resolved { "☑" } else { "☐" };
                let check_color = if item.resolved {
                    Color::Green
                } else {
                    Color::DarkGray
                };
                let name_style = if item.resolved {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(THEME.fg)
                };

                let mut spans = vec![
                    Span::raw("      "),
                    Span::styled(format!("{check} "), Style::default().fg(check_color)),
                    Span::styled(item.name.clone(), name_style),
                ];

                if let Some(assignee) = &item.assignee
                    && !assignee.username.is_empty()
                {
                    spans.push(Span::styled(
                        format!("  @{}", assignee.username),
                        Style::default().fg(Color::Cyan),
                    ));
                }

                lines.push(Line::from(spans));
            }
        }
    }

    // ─── Linked Tasks ───
    if !task.linked_tasks.is_empty() {
        lines.push(Line::from(""));
        lines.push(section_header(&format!(
            "Linked Tasks ({})",
            task.linked_tasks.len()
        )));
        lines.push(Line::from(""));

        for lt in &task.linked_tasks {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled("🔗 ", Style::default()),
                Span::styled(
                    lt.task_id.clone(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::UNDERLINED),
                ),
            ]));
        }
    }

    // ─── Dependencies ───
    if !task.dependencies.is_empty() {
        lines.push(Line::from(""));
        lines.push(section_header(&format!(
            "Dependencies ({})",
            task.dependencies.len()
        )));
        lines.push(Line::from(""));

        for dep in &task.dependencies {
            let (icon, label) = if dep.depends_on == task.id {
                ("→", format!("blocks {}", dep.task_id))
            } else {
                ("←", format!("waiting on {}", dep.depends_on))
            };
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(format!("{icon} "), Style::default().fg(Color::Yellow)),
                Span::styled(label, Style::default().fg(THEME.fg)),
            ]));
        }
    }

    // ─── Attachments ───
    if !task.attachments.is_empty() {
        lines.push(Line::from(""));
        lines.push(section_header(&format!(
            "Attachments ({})",
            task.attachments.len()
        )));
        lines.push(Line::from(""));

        for att in &task.attachments {
            let name = att.title.as_deref().unwrap_or("Untitled");
            let ext = att.extension.as_deref().unwrap_or("");
            let icon = match ext {
                "pdf" => "📄",
                "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => "🖼️",
                "zip" | "tar" | "gz" | "rar" => "📦",
                "mp4" | "mov" | "avi" => "🎬",
                "mp3" | "wav" | "ogg" => "🎵",
                _ => "📎",
            };
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(format!("{icon} "), Style::default()),
                Span::styled(name.to_string(), Style::default().fg(THEME.fg)),
            ]));
        }
    }

    lines.push(Line::from(""));

    // Apply scroll offset.
    let scroll = app.scroll_offset as usize;
    let content = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));

    frame.render_widget(content, inner);
}

fn meta_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {label}: "),
            Style::default()
                .fg(THEME.muted)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(value.to_string(), Style::default().fg(THEME.fg)),
    ])
}

fn priority_emoji(p: &str) -> String {
    match p {
        "urgent" => "🔴 Urgent",
        "high" => "🟠 High",
        "normal" => "🟡 Normal",
        "low" => "🔵 Low",
        _ => "⚪ None",
    }
    .to_string()
}

fn format_timestamp(ts: Option<&str>) -> Option<String> {
    let ms: i64 = ts?.parse().ok()?;
    let dt = chrono::DateTime::from_timestamp_millis(ms)?;
    Some(dt.format("%b %d, %Y").to_string())
}

fn section_header(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  ── {title} ──────────────"),
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    ))
}

fn format_duration(ms: u64) -> String {
    let total_secs = ms / 1000;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    if hours > 0 && minutes > 0 {
        format!("{hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h")
    } else {
        format!("{minutes}m")
    }
}

fn format_custom_field_value(field_type: &str, value: Option<&serde_json::Value>) -> String {
    let Some(val) = value else {
        return "—".to_string();
    };
    match field_type {
        "number" | "currency" => {
            if let Some(n) = val.as_f64() {
                if n.fract() == 0.0_f64 {
                    format!("{}", n as i64)
                } else {
                    format!("{n:.2}")
                }
            } else {
                val.to_string()
            }
        }
        "checkbox" => if val.as_bool().unwrap_or(false) {
            "✅"
        } else {
            "☐"
        }
        .to_string(),
        "date" => {
            if let Some(s) = val.as_str() {
                format_timestamp(Some(s)).unwrap_or_else(|| s.to_string())
            } else if let Some(n) = val.as_i64() {
                let s = n.to_string();
                format_timestamp(Some(&s)).unwrap_or_else(|| val.to_string())
            } else {
                val.to_string()
            }
        }
        "drop_down" | "labels" => {
            if let Some(arr) = val.as_array() {
                arr.iter()
                    .filter_map(|v: &serde_json::Value| {
                        v.get("name")
                            .or(v.get("label"))
                            .and_then(|n: &serde_json::Value| n.as_str())
                    })
                    .collect::<Vec<&str>>()
                    .join(", ")
            } else if let Some(obj) = val.as_object() {
                obj.get("name")
                    .or(obj.get("label"))
                    .and_then(|n: &serde_json::Value| n.as_str())
                    .unwrap_or("—")
                    .to_string()
            } else {
                val.as_str()
                    .map(String::from)
                    .unwrap_or_else(|| val.to_string())
            }
        }
        "url" | "email" | "phone" | "short_text" | "text" => {
            val.as_str().unwrap_or("—").to_string()
        }
        _ => val
            .as_str()
            .map(String::from)
            .unwrap_or_else(|| val.to_string()),
    }
}
