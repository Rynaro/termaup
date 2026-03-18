use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::app::App;
use crate::theme::{THEME, hex_to_color};

/// Renders the workspace selector screen.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Select Workspace ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(THEME.border_style());

    if app.workspaces.is_empty() {
        let paragraph = Paragraph::new("  No workspaces available.")
            .block(block)
            .style(THEME.muted_style());
        frame.render_widget(paragraph, area);
        return;
    }

    let indices = app.filtered_workspace_indices();

    if indices.is_empty() {
        let paragraph = Paragraph::new("  No matching workspaces.")
            .block(block)
            .style(THEME.muted_style());
        frame.render_widget(paragraph, area);
        return;
    }

    let lines: Vec<Line> = indices
        .iter()
        .enumerate()
        .map(|(display_idx, &real_idx)| {
            let ws = &app.workspaces[real_idx];
            let selected = display_idx == app.selected_index;
            let marker = if selected { " ▸ " } else { "   " };
            let member_count = ws.members.len();
            let members_label = if member_count == 1 {
                "1 member".to_string()
            } else {
                format!("{member_count} members")
            };

            let avatar_color = ws.color.as_deref().map(hex_to_color).unwrap_or(THEME.muted);

            let style = if selected {
                THEME.selected_style()
            } else {
                THEME.normal_style()
            };

            Line::from(vec![
                Span::styled(marker, style),
                Span::styled("● ", Style::default().fg(avatar_color)),
                Span::styled(&ws.name, style),
                Span::styled(format!("  ({members_label})"), THEME.muted_style()),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
