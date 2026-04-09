use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};

use crate::app::{App, Screen};
use crate::theme::THEME;

/// Renders a centred help overlay with context-aware keybindings.
pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let width = 54u16.min(area.width.saturating_sub(4));
    let height = 30u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup = Rect::new(x, y, width, height);

    // Dim background.
    let dim = Paragraph::new("").style(Style::default().bg(Color::Rgb(10, 10, 10)));
    frame.render_widget(dim, area);
    frame.render_widget(Clear, popup);

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Keyboard Shortcuts",
            Style::default()
                .fg(THEME.header_fg)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    // Global keys.
    lines.push(section_header(" Global"));
    lines.push(key_line("q / Ctrl+C", "Quit"));
    lines.push(key_line("?", "Toggle this help"));
    lines.push(key_line("/", "Search / filter"));
    lines.push(Line::from(""));

    // Screen-specific keys.
    match app.screen {
        Screen::Loading => {}
        Screen::WorkspaceSelect => {
            lines.push(section_header(" Workspace Select"));
            lines.push(key_line("↑/k  ↓/j", "Navigate"));
            lines.push(key_line("Enter", "Select workspace"));
            lines.push(key_line("r", "Refresh"));
        }
        Screen::SpaceList => {
            lines.push(section_header(" Space List"));
            lines.push(key_line("↑/k  ↓/j", "Navigate"));
            lines.push(key_line("Enter", "Select space"));
            lines.push(key_line("Esc", "Back to workspaces"));
            lines.push(key_line("r", "Refresh"));
        }
        Screen::SpaceContent => {
            lines.push(section_header(" Space Contents"));
            lines.push(key_line("↑/k  ↓/j", "Navigate"));
            lines.push(key_line("Enter", "Open folder / list"));
            lines.push(key_line("Esc", "Back to spaces"));
            lines.push(key_line("r", "Refresh"));
        }
        Screen::TaskList => {
            lines.push(section_header(" Task List"));
            lines.push(key_line("↑/k  ↓/j", "Navigate"));
            lines.push(key_line("Tab/S-Tab", "Switch status group"));
            lines.push(key_line("Enter", "View task detail"));
            lines.push(key_line("Esc", "Back to contents"));
            lines.push(key_line("v", "Cycle view mode"));
            lines.push(key_line("f", "Open filter panel"));
            lines.push(key_line("m", "Toggle Me Mode"));
            lines.push(key_line("r", "Refresh"));
        }
        Screen::TaskDetail => {
            lines.push(section_header(" Task Detail"));
            lines.push(key_line("↑/k  ↓/j", "Scroll"));
            lines.push(key_line("c", "Toggle comments"));
            lines.push(key_line("Esc", "Back to tasks"));
            lines.push(Line::from(""));
            lines.push(section_header(" Comment Sidebar"));
            lines.push(key_line("↑/k  ↓/j", "Navigate comments"));
            lines.push(key_line("n", "New comment"));
            lines.push(key_line("r", "Reply"));
            lines.push(key_line("e", "Edit comment"));
            lines.push(key_line("d", "Delete comment"));
            lines.push(key_line("c", "Close sidebar"));
            lines.push(Line::from(""));
            lines.push(section_header(" Composing"));
            lines.push(key_line("Enter", "Send comment"));
            lines.push(key_line("Alt+Enter", "Insert newline"));
            lines.push(key_line("Esc", "Cancel"));
        }
    }

    lines.push(Line::from(""));

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.header_fg));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, popup);
}

fn section_header(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  ─{title}─────────────────"),
        Style::default()
            .fg(THEME.muted)
            .add_modifier(Modifier::BOLD),
    ))
}

fn key_line(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {key:<14}"),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(desc.to_string(), Style::default().fg(THEME.muted)),
    ])
}
