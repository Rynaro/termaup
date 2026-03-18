use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::app::App;
use crate::theme::{THEME, hex_to_color};

/// Minimum column width for board cards.
const MIN_COL_WIDTH: u16 = 20;
/// Maximum column width for board cards.
const MAX_COL_WIDTH: u16 = 30;

/// Renders the Vision Board view — horizontal kanban columns by status.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let list_name = app
        .current_list
        .as_ref()
        .map(|l| l.name.as_str())
        .unwrap_or("Tasks");

    let outer_block = Block::default()
        .title(format!(" {list_name} — Vision (Board) "))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(THEME.border_style());

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

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

    let num_groups = app.status_groups.len();
    if num_groups == 0 {
        return;
    }

    // Calculate column widths — distribute evenly, within min/max bounds.
    let available_width = inner.width;
    let col_width = (available_width / num_groups as u16).clamp(MIN_COL_WIDTH, MAX_COL_WIDTH);
    let visible_cols = (available_width / col_width) as usize;
    let visible_cols = visible_cols.min(num_groups);

    // Horizontal scroll: ensure focused group is visible.
    let h_scroll = if app.focused_group >= visible_cols {
        app.focused_group - visible_cols + 1
    } else {
        0
    };

    let constraints: Vec<Constraint> = (0..visible_cols)
        .map(|_| Constraint::Length(col_width))
        .collect();

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(inner);

    for (col_idx, col_area) in columns.iter().enumerate() {
        let group_idx = h_scroll + col_idx;
        if group_idx >= num_groups {
            break;
        }
        let is_focused = group_idx == app.focused_group;

        render_column(app, frame, *col_area, group_idx, is_focused);
    }

    // Loading indicator.
    if app.loading_more {
        let loading_text = format!(" {} Loading… ", app.spinner_char());
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
    }
}

fn render_column(app: &App, frame: &mut Frame, area: Rect, group_idx: usize, is_focused: bool) {
    let group = &app.status_groups[group_idx];
    let status_color = hex_to_color(&group.color);
    let count = group.task_indices.len();

    let border_style = if is_focused {
        Style::default().fg(status_color)
    } else {
        THEME.border_style()
    };

    let title_style = if is_focused {
        Style::default()
            .fg(status_color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(THEME.muted)
    };

    let title = format!(" {} ({count}) ", group.name);

    let block = Block::default()
        .title(Span::styled(title, title_style))
        .borders(Borders::ALL)
        .border_type(if is_focused {
            BorderType::Thick
        } else {
            BorderType::Rounded
        })
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if count == 0 {
        let msg = Paragraph::new("  (empty)").style(THEME.muted_style());
        frame.render_widget(msg, inner);
        return;
    }

    let visible_height = inner.height as usize;

    // Vertical scroll within column.
    let v_scroll = if is_focused && app.group_selected_index >= visible_height {
        app.group_selected_index - visible_height + 1
    } else {
        0
    };

    let lines: Vec<Line> = group
        .task_indices
        .iter()
        .enumerate()
        .skip(v_scroll)
        .take(visible_height)
        .map(|(item_idx, &task_idx)| {
            let selected = is_focused && item_idx == app.group_selected_index;
            render_card(app, task_idx, selected, inner.width as usize)
        })
        .collect();

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_card(app: &App, task_idx: usize, selected: bool, width: usize) -> Line<'static> {
    let task = &app.tasks[task_idx];
    let style = if selected {
        THEME.selected_style()
    } else {
        THEME.normal_style()
    };

    let marker = if selected { "▸" } else { " " };

    let priority = match &task.priority {
        Some(p) => match p.priority.as_deref() {
            Some("urgent") => "🔴",
            Some("high") => "🟠",
            Some("normal") => "🟡",
            Some("low") => "🔵",
            _ => "",
        },
        None => "",
    };

    // Truncate name to fit column width, accounting for marker + priority.
    let max_name = width.saturating_sub(5);
    let name = if task.name.len() > max_name {
        format!("{}…", &task.name[..max_name.saturating_sub(1)])
    } else {
        task.name.clone()
    };

    Line::from(vec![
        Span::styled(format!("{marker} "), style),
        Span::styled(format!("{priority} "), Style::default()),
        Span::styled(name, style),
    ])
}
