use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::app::App;
use crate::theme::{THEME, hex_to_color};

/// Renders the Vision Sections view — tasks grouped by status as vertical sections.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let list_name = app
        .current_list
        .as_ref()
        .map(|l| l.name.as_str())
        .unwrap_or("Tasks");

    let block = Block::default()
        .title(format!(" {list_name} — Vision (Sections) "))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(THEME.border_style());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.status_groups.is_empty() {
        let msg = if app.loading {
            "  Loading tasks…"
        } else if app.task_filters.is_active() {
            "  No tasks match the active filters."
        } else {
            "  No tasks in this list."
        };
        let paragraph = Paragraph::new(msg).style(THEME.muted_style());
        frame.render_widget(paragraph, inner);
        return;
    }

    // Build all lines: status headers + task rows.
    let mut all_lines: Vec<SectionLine> = Vec::new();
    for (group_idx, group) in app.status_groups.iter().enumerate() {
        // Status header.
        let count = group.task_indices.len();
        let header_focused = group_idx == app.focused_group;
        all_lines.push(SectionLine::Header {
            name: group.name.clone(),
            color: group.color.clone(),
            count,
            focused: header_focused,
        });

        // Task rows within this group.
        for (item_idx, &task_idx) in group.task_indices.iter().enumerate() {
            let selected = group_idx == app.focused_group && item_idx == app.group_selected_index;
            all_lines.push(SectionLine::Task { task_idx, selected });
        }

        // Spacer between groups.
        if group_idx < app.status_groups.len() - 1 {
            all_lines.push(SectionLine::Spacer);
        }
    }

    let visible_height = inner.height as usize;

    // Calculate scroll to keep focused item visible.
    let focus_line = find_focus_line(&all_lines, app.focused_group, app.group_selected_index);
    let scroll = if focus_line >= visible_height {
        focus_line - visible_height + 1
    } else {
        0
    };

    let lines: Vec<Line> = all_lines
        .iter()
        .skip(scroll)
        .take(visible_height)
        .map(|sl| match sl {
            SectionLine::Header {
                name,
                color,
                count,
                focused,
            } => render_header(name, color, *count, *focused),
            SectionLine::Task { task_idx, selected } => render_task_row(app, *task_idx, *selected),
            SectionLine::Spacer => Line::from(""),
        })
        .collect();

    frame.render_widget(Paragraph::new(lines), inner);

    // Scroll indicator.
    let total_tasks: usize = app.status_groups.iter().map(|g| g.task_indices.len()).sum();
    if all_lines.len() > visible_height {
        let indicator = format!(" {total_tasks} tasks ");
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

    // Loading more indicator.
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

enum SectionLine {
    Header {
        name: String,
        color: String,
        count: usize,
        focused: bool,
    },
    Task {
        task_idx: usize,
        selected: bool,
    },
    Spacer,
}

fn find_focus_line(lines: &[SectionLine], focused_group: usize, group_item: usize) -> usize {
    let mut current_group = 0;
    let mut item_in_group = 0;

    for (i, line) in lines.iter().enumerate() {
        match line {
            SectionLine::Header { .. } => {
                // If this is the focused group header and no items selected yet,
                // this is the focus position.
                if current_group == focused_group && group_item == 0 {
                    // Return position of first task if exists, else header.
                    if let Some(SectionLine::Task { .. }) = lines.get(i + 1) {
                        return i + 1;
                    }
                    return i;
                }
            }
            SectionLine::Task { .. } => {
                if current_group == focused_group && item_in_group == group_item {
                    return i;
                }
                item_in_group += 1;
            }
            SectionLine::Spacer => {
                current_group += 1;
                item_in_group = 0;
            }
        }
    }
    0
}

fn render_header(name: &str, color: &str, count: usize, focused: bool) -> Line<'static> {
    let status_color = hex_to_color(color);
    let marker = if focused { "▸ " } else { "  " };
    let style = if focused {
        Style::default()
            .fg(THEME.header_fg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(THEME.fg).add_modifier(Modifier::BOLD)
    };

    Line::from(vec![
        Span::styled(marker.to_string(), style),
        Span::styled("█ ", Style::default().fg(status_color)),
        Span::styled(format!("{name}  "), style),
        Span::styled(format!("({count})"), Style::default().fg(THEME.muted)),
    ])
}

fn render_task_row(app: &App, task_idx: usize, selected: bool) -> Line<'static> {
    let Some(task) = app.tasks.get(task_idx) else {
        return Line::from(Span::styled(
            "  (stale reference)",
            Style::default().fg(Color::DarkGray),
        ));
    };
    let style = if selected {
        THEME.selected_style()
    } else {
        THEME.normal_style()
    };

    let marker = if selected { "  ▸ " } else { "    " };

    let assignee = task
        .assignees
        .first()
        .map(|u| u.initials.as_deref().unwrap_or(&u.username).to_string())
        .unwrap_or_default();

    let priority = match &task.priority {
        Some(p) => match p.priority.as_deref() {
            Some("urgent") => "🔴",
            Some("high") => "🟠",
            Some("normal") => "🟡",
            Some("low") => "🔵",
            _ => "⚪",
        },
        None => "⚪",
    };

    let name = if task.name.len() > 36 {
        format!("{}…", &task.name[..35])
    } else {
        task.name.clone()
    };

    Line::from(vec![
        Span::styled(marker.to_string(), style),
        Span::styled(format!("{priority} "), Style::default()),
        Span::styled(format!("{name:<38}"), style),
        Span::styled(format!("{assignee:<12}"), style),
    ])
}
