use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::app::App;
use crate::theme::{THEME, hex_to_color};

/// Renders the task list for a single list.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let list_name = app
        .current_list
        .as_ref()
        .map(|l| l.name.as_str())
        .unwrap_or("Tasks");

    let block = Block::default()
        .title(format!(" {list_name} — Tasks "))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(THEME.border_style());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.tasks.is_empty() {
        let msg = if app.loading {
            "  Loading tasks…"
        } else {
            "  No tasks in this list."
        };
        let paragraph = Paragraph::new(msg).style(THEME.muted_style());
        frame.render_widget(paragraph, inner);
        return;
    }

    let indices = app.filtered_task_indices();

    if indices.is_empty() {
        let msg = if app.task_filters.is_active() {
            "  No tasks match the active filters."
        } else if !app.filter_text.is_empty() {
            "  No matching tasks."
        } else {
            "  No tasks to display."
        };
        let paragraph = Paragraph::new(msg).style(THEME.muted_style());
        frame.render_widget(paragraph, inner);
        return;
    }

    // Header row.
    let header_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    let header_style = Style::default()
        .fg(THEME.muted)
        .add_modifier(Modifier::BOLD);

    let header = Line::from(vec![
        Span::styled(" ●  ", header_style),
        Span::styled(pad_right("Task", 40), header_style),
        Span::styled(pad_right("Assignee", 14), header_style),
        Span::styled(pad_right("Priority", 10), header_style),
        Span::styled("Due Date", header_style),
    ]);
    frame.render_widget(Paragraph::new(header), header_layout[0]);

    let visible_height = header_layout[1].height as usize;
    let total = indices.len();

    // Keep selected item visible.
    let scroll = if app.selected_index >= visible_height {
        app.selected_index - visible_height + 1
    } else {
        0
    };

    let lines: Vec<Line> = indices
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_height)
        .map(|(display_idx, &real_idx)| {
            let task = &app.tasks[real_idx];
            let selected = display_idx == app.selected_index;
            let row_style = if selected {
                THEME.selected_style()
            } else {
                THEME.normal_style()
            };

            let marker = if selected { " ▸ " } else { "   " };

            let status_color = hex_to_color(&task.status.color);

            let assignee = task
                .assignees
                .first()
                .map(|u| u.initials.as_deref().unwrap_or(&u.username).to_string())
                .unwrap_or_default();

            let priority = format_priority_emoji(&task.priority);
            let due = format_relative_date(task.due_date.as_deref());

            let name = truncate(&task.name, 38);

            Line::from(vec![
                Span::styled(marker, row_style),
                Span::styled("● ", Style::default().fg(status_color)),
                Span::styled(pad_right(&name, 40), row_style),
                Span::styled(pad_right(&assignee, 14), row_style),
                Span::styled(pad_right(&priority, 10), row_style),
                Span::styled(due.0, due.1),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, header_layout[1]);

    // Scroll indicator.
    if total > visible_height {
        let indicator = format!(" {}/{} ", app.selected_index + 1, total);
        let indicator_area = Rect {
            x: area.x + area.width - indicator.len() as u16 - 2,
            y: area.y + area.height - 1,
            width: indicator.len() as u16,
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(indicator).style(THEME.muted_style()),
            indicator_area,
        );
    }

    // "Loading more..." indicator when fetching next page.
    if app.loading_more {
        let loading_text = format!(" {} Loading more… ", app.spinner_char());
        let loading_area = Rect {
            x: area.x + 1,
            y: area.y + area.height - 1,
            width: loading_text.len() as u16,
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(loading_text).style(Style::default().fg(THEME.loading_fg)),
            loading_area,
        );
    } else if app.has_more_pages {
        let more_text = " ↓ more ";
        let more_area = Rect {
            x: area.x + 1,
            y: area.y + area.height - 1,
            width: more_text.len() as u16,
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(more_text).style(THEME.muted_style()),
            more_area,
        );
    }
}

fn format_priority_emoji(priority: &Option<clickup_api::models::TaskPriority>) -> String {
    match priority {
        Some(p) => match p.priority.as_deref() {
            Some("urgent") => "🔴".to_string(),
            Some("high") => "🟠".to_string(),
            Some("normal") => "🟡".to_string(),
            Some("low") => "🔵".to_string(),
            _ => "⚪".to_string(),
        },
        None => "⚪".to_string(),
    }
}

fn format_relative_date(ts: Option<&str>) -> (String, Style) {
    let Some(ts_str) = ts else {
        return ("—".to_string(), Style::default().fg(Color::DarkGray));
    };

    let Ok(ms) = ts_str.parse::<i64>() else {
        return (ts_str.to_string(), Style::default().fg(Color::DarkGray));
    };

    let Some(dt) = chrono::DateTime::from_timestamp_millis(ms) else {
        return ("—".to_string(), Style::default().fg(Color::DarkGray));
    };

    let now = chrono::Utc::now();
    let diff = dt.signed_duration_since(now);
    let days = diff.num_days();

    if days == 0 {
        ("today".to_string(), Style::default().fg(Color::Yellow))
    } else if days > 0 {
        (format!("in {days}d"), Style::default().fg(Color::Green))
    } else {
        let abs = days.unsigned_abs();
        (format!("{abs}d ago"), Style::default().fg(Color::Red))
    }
}

fn pad_right(s: &str, width: usize) -> String {
    if s.len() >= width {
        s[..width].to_string()
    } else {
        format!("{s:<width$}")
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}
