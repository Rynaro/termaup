use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::App;
use crate::theme::THEME;

/// Renders the inline search bar at the given area (typically 1 row).
pub fn render_search_bar(app: &App, frame: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled(" / ", Style::default().fg(Color::Black).bg(THEME.search_fg)),
        Span::styled(
            format!(" {} ", app.filter_text),
            Style::default().fg(THEME.search_fg).bg(THEME.search_bg),
        ),
        Span::styled(
            "▏",
            Style::default()
                .fg(THEME.search_fg)
                .add_modifier(Modifier::SLOW_BLINK),
        ),
        Span::styled(
            "  Enter confirm  Esc cancel",
            Style::default().fg(THEME.muted),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}
