use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Splits the terminal area into the standard three-row layout:
/// top bar (1 line), main content, bottom bar (1 line).
#[allow(dead_code)]
pub fn main_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // top bar
            Constraint::Min(1),    // main content
            Constraint::Length(1), // bottom bar
        ])
        .split(area)
        .to_vec()
}
