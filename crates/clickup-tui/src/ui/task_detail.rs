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
            lines.push(Line::from(Span::styled(
                "  ── Description ──────────────",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )));
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
        lines.push(Line::from(Span::styled(
            "  ── Description ──────────────",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )));
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
        lines.push(Line::from(Span::styled(
            format!("  ── Subtasks ({}) ─────────────", subtasks.len()),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )));
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

    // ─── Comments ───
    if !app.comments.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  ── Comments ({}) ─────────────", app.comments.len()),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        for comment in &app.comments {
            let date = format_timestamp(Some(&comment.date)).unwrap_or_default();

            lines.push(Line::from(vec![
                Span::styled(
                    format!("    {} ", comment.user.username),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(date, Style::default().fg(Color::DarkGray)),
            ]));

            for text_line in comment.comment_text.lines() {
                lines.push(Line::from(format!("      {text_line}")));
            }
            lines.push(Line::from(""));
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
