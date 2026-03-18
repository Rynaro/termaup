use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::app::App;
use crate::theme::{THEME, hex_to_color};

/// Renders the space list screen.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let ws_name = app
        .current_workspace
        .as_ref()
        .map(|w| w.name.as_str())
        .unwrap_or("Workspace");

    let block = Block::default()
        .title(format!(" Spaces — {ws_name} "))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(THEME.border_style());

    if app.spaces.is_empty() {
        let paragraph = Paragraph::new("  No spaces found.")
            .block(block)
            .style(THEME.muted_style());
        frame.render_widget(paragraph, area);
        return;
    }

    let indices = app.filtered_space_indices();

    if indices.is_empty() {
        let paragraph = Paragraph::new("  No matching spaces.")
            .block(block)
            .style(THEME.muted_style());
        frame.render_widget(paragraph, area);
        return;
    }

    let lines: Vec<Line> = indices
        .iter()
        .enumerate()
        .map(|(display_idx, &real_idx)| {
            let space = &app.spaces[real_idx];
            let selected = display_idx == app.selected_index;
            let marker = if selected { " ▸ " } else { "   " };

            let private_indicator = if space.private { " 🔒" } else { "" };

            let status_count = space.statuses.len();
            let statuses_label = if status_count == 1 {
                "1 status".to_string()
            } else {
                format!("{status_count} statuses")
            };

            let accent = space.color.as_deref().map(hex_to_color).unwrap_or(THEME.fg);

            let style = if selected {
                THEME.selected_style()
            } else {
                THEME.normal_style()
            };

            Line::from(vec![
                Span::styled(marker, style),
                Span::styled("● ", Style::default().fg(accent)),
                Span::styled(&space.name, style),
                Span::styled(private_indicator, Style::default()),
                Span::styled(format!("  ({statuses_label})"), THEME.muted_style()),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
