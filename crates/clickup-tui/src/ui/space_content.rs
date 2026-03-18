use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::app::{App, SpaceContentItem};
use crate::theme::THEME;

/// Renders the space content screen showing folders and lists.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let space_name = app
        .current_space
        .as_ref()
        .map(|s| s.name.as_str())
        .unwrap_or("Space");

    let block = Block::default()
        .title(format!(" {space_name} — Contents "))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(THEME.border_style());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.space_content.is_empty() {
        let msg = Paragraph::new("  No folders or lists found.").style(THEME.muted_style());
        frame.render_widget(msg, inner);
        return;
    }

    let indices = app.filtered_content_indices();

    if indices.is_empty() {
        let msg = Paragraph::new("  No matching items.").style(THEME.muted_style());
        frame.render_widget(msg, inner);
        return;
    }

    let visible_height = inner.height as usize;
    let total = indices.len();

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
            let item = &app.space_content[real_idx];
            let selected = display_idx == app.selected_index;
            let row_style = if selected {
                THEME.selected_style()
            } else {
                THEME.normal_style()
            };

            let marker = if selected { " ▸ " } else { "   " };

            match item {
                SpaceContentItem::FolderItem {
                    folder, expanded, ..
                } => {
                    let arrow = if *expanded { "▾" } else { "▸" };
                    let list_count = folder.lists.len();
                    let count_label = if list_count == 1 {
                        "1 list".to_string()
                    } else {
                        format!("{list_count} lists")
                    };

                    Line::from(vec![
                        Span::styled(marker, row_style),
                        Span::styled(format!("📁 {arrow} "), Style::default().fg(THEME.folder_fg)),
                        Span::styled(folder.name.clone(), row_style),
                        Span::styled(
                            format!("  ({count_label})"),
                            Style::default().fg(THEME.muted),
                        ),
                    ])
                }
                SpaceContentItem::ListInFolder { list, .. } => {
                    let task_count = list.task_count.as_deref().unwrap_or("?");
                    let count_label = format!("{task_count} tasks");

                    Line::from(vec![
                        Span::styled(marker, row_style),
                        Span::styled("     📋 ", Style::default().fg(THEME.list_fg)),
                        Span::styled(list.name.clone(), row_style),
                        Span::styled(
                            format!("  ({count_label})"),
                            Style::default().fg(THEME.muted),
                        ),
                    ])
                }
                SpaceContentItem::FolderlessList { list } => {
                    let task_count = list.task_count.as_deref().unwrap_or("?");
                    let count_label = format!("{task_count} tasks");

                    Line::from(vec![
                        Span::styled(marker, row_style),
                        Span::styled("📋 ", Style::default().fg(THEME.list_fg)),
                        Span::styled(list.name.clone(), row_style),
                        Span::styled(
                            format!("  ({count_label})"),
                            Style::default().fg(THEME.muted),
                        ),
                    ])
                }
            }
        })
        .collect();

    let paragraph = Paragraph::new(lines);

    // Split area: content + scroll indicator
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(inner);

    frame.render_widget(paragraph, layout[0]);

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
}
