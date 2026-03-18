use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};

use clickup_api::filter_config::DueDateFilter;

use crate::app::App;
use crate::theme::{THEME, hex_to_color};

/// Index of the currently focused filter section in the panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterSection {
    /// Status filter (multi-select).
    #[default]
    Statuses,
    /// Priority filter (multi-select).
    Priorities,
    /// Due date filter (radio select).
    DueDate,
    /// Me Mode toggle.
    MeMode,
    /// Include closed toggle.
    IncludeClosed,
}

impl FilterSection {
    /// Moves to the next section.
    pub fn next(self) -> Self {
        match self {
            Self::Statuses => Self::Priorities,
            Self::Priorities => Self::DueDate,
            Self::DueDate => Self::MeMode,
            Self::MeMode => Self::IncludeClosed,
            Self::IncludeClosed => Self::Statuses,
        }
    }

    /// Moves to the previous section.
    pub fn prev(self) -> Self {
        match self {
            Self::Statuses => Self::IncludeClosed,
            Self::Priorities => Self::Statuses,
            Self::DueDate => Self::Priorities,
            Self::MeMode => Self::DueDate,
            Self::IncludeClosed => Self::MeMode,
        }
    }
}

/// State for the filter panel overlay.
#[derive(Debug, Clone, Default)]
pub struct FilterPanelState {
    /// Which section is currently focused.
    pub section: FilterSection,
    /// Cursor within the current section's items.
    pub cursor: usize,
}

impl FilterPanelState {
    /// Resets the panel state.
    pub fn reset(&mut self) {
        self.section = FilterSection::default();
        self.cursor = 0;
    }

    /// Moves focus to the next section.
    pub fn next_section(&mut self) {
        self.section = self.section.next();
        self.cursor = 0;
    }

    /// Moves focus to the previous section.
    pub fn prev_section(&mut self) {
        self.section = self.section.prev();
        self.cursor = 0;
    }
}

/// Renders the filter panel overlay.
pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let width = 48u16.min(area.width.saturating_sub(4));
    let height = (area.height - 4).min(30);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup = Rect::new(x, y, width, height);

    // Dim background.
    let dim = Paragraph::new("").style(Style::default().bg(Color::Rgb(10, 10, 10)));
    frame.render_widget(dim, area);
    frame.render_widget(Clear, popup);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    let panel = &app.filter_panel_state;

    // --- Status section ---
    lines.push(section_header(
        "Status",
        panel.section == FilterSection::Statuses,
    ));
    let statuses = available_statuses(app);
    if statuses.is_empty() {
        lines.push(Line::from(Span::styled(
            "    (no statuses available)",
            Style::default().fg(THEME.muted),
        )));
    } else {
        for (i, (name, color)) in statuses.iter().enumerate() {
            let selected = app
                .task_filters
                .statuses
                .iter()
                .any(|s| s.eq_ignore_ascii_case(name));
            let focused = panel.section == FilterSection::Statuses && panel.cursor == i;
            lines.push(checkbox_line(name, selected, focused, Some(color)));
        }
    }
    lines.push(Line::from(""));

    // --- Priority section ---
    lines.push(section_header(
        "Priority",
        panel.section == FilterSection::Priorities,
    ));
    let priorities = ["urgent", "high", "normal", "low", "none"];
    let emojis = ["🔴", "🟠", "🟡", "🔵", "⚪"];
    for (i, (prio, emoji)) in priorities.iter().zip(emojis.iter()).enumerate() {
        let selected = app
            .task_filters
            .priorities
            .iter()
            .any(|p| p.eq_ignore_ascii_case(prio));
        let focused = panel.section == FilterSection::Priorities && panel.cursor == i;
        let label = format!("{emoji} {prio}");
        lines.push(checkbox_line(&label, selected, focused, None));
    }
    lines.push(Line::from(""));

    // --- Due date section ---
    lines.push(section_header(
        "Due Date",
        panel.section == FilterSection::DueDate,
    ));
    let due_options = [
        (DueDateFilter::All, "All"),
        (DueDateFilter::Overdue, "Overdue"),
        (DueDateFilter::Today, "Today"),
        (DueDateFilter::ThisWeek, "This week"),
        (DueDateFilter::NoDueDate, "No due date"),
    ];
    for (i, (variant, label)) in due_options.iter().enumerate() {
        let selected = app.task_filters.due_date_filter == *variant;
        let focused = panel.section == FilterSection::DueDate && panel.cursor == i;
        lines.push(radio_line(label, selected, focused));
    }
    lines.push(Line::from(""));

    // --- Me Mode toggle ---
    lines.push(section_header(
        "Me Mode",
        panel.section == FilterSection::MeMode,
    ));
    let me_label = if let Some(user) = &app.current_user {
        format!("Show only my tasks ({})", user.username)
    } else {
        "Show only my tasks".to_string()
    };
    let me_focused = panel.section == FilterSection::MeMode;
    lines.push(toggle_line(&me_label, app.task_filters.me_mode, me_focused));
    lines.push(Line::from(""));

    // --- Include Closed toggle ---
    lines.push(section_header(
        "Closed Tasks",
        panel.section == FilterSection::IncludeClosed,
    ));
    let closed_focused = panel.section == FilterSection::IncludeClosed;
    lines.push(toggle_line(
        "Include closed tasks",
        app.task_filters.include_closed,
        closed_focused,
    ));

    let title = if app.task_filters.is_active() {
        " Filters (active) "
    } else {
        " Filters "
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.header_fg));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, popup);

    // Key hints at bottom of panel.
    let hints = " Tab:section  ↑↓:move  Space:toggle  Esc:close ";
    let hints_area = Rect {
        x: popup.x + 1,
        y: popup.y + popup.height - 1,
        width: hints.len() as u16,
        height: 1,
    };
    if hints_area.x + hints_area.width <= area.width {
        frame.render_widget(Paragraph::new(hints).style(THEME.muted_style()), hints_area);
    }
}

/// Returns available statuses from the current space.
fn available_statuses(app: &App) -> Vec<(String, String)> {
    app.current_space
        .as_ref()
        .map(|space| {
            space
                .statuses
                .iter()
                .map(|s| (s.status.clone(), s.color.clone()))
                .collect()
        })
        .unwrap_or_default()
}

fn section_header(title: &str, focused: bool) -> Line<'static> {
    let style = if focused {
        Style::default()
            .fg(THEME.header_fg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(THEME.muted)
            .add_modifier(Modifier::BOLD)
    };
    let marker = if focused { "▸" } else { " " };
    Line::from(Span::styled(format!("  {marker} {title}"), style))
}

fn checkbox_line(label: &str, checked: bool, focused: bool, color: Option<&str>) -> Line<'static> {
    let check = if checked { "☑" } else { "☐" };
    let focus_marker = if focused { "›" } else { " " };

    let style = if focused {
        THEME.selected_style()
    } else {
        THEME.normal_style()
    };

    let mut spans = vec![Span::styled(format!("    {focus_marker} {check} "), style)];

    if let Some(hex) = color {
        spans.push(Span::styled("● ", Style::default().fg(hex_to_color(hex))));
    }

    spans.push(Span::styled(label.to_string(), style));
    Line::from(spans)
}

fn radio_line(label: &str, selected: bool, focused: bool) -> Line<'static> {
    let radio = if selected { "◉" } else { "○" };
    let focus_marker = if focused { "›" } else { " " };
    let style = if focused {
        THEME.selected_style()
    } else {
        THEME.normal_style()
    };
    Line::from(Span::styled(
        format!("    {focus_marker} {radio} {label}"),
        style,
    ))
}

fn toggle_line(label: &str, on: bool, focused: bool) -> Line<'static> {
    let toggle = if on { "[ON] " } else { "[OFF]" };
    let focus_marker = if focused { "›" } else { " " };
    let style = if focused {
        THEME.selected_style()
    } else {
        THEME.normal_style()
    };
    let toggle_color = if on {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(THEME.muted)
    };
    Line::from(vec![
        Span::styled(format!("    {focus_marker} "), style),
        Span::styled(toggle.to_string(), toggle_color),
        Span::styled(format!(" {label}"), style),
    ])
}

/// Returns the number of items in a section for cursor bounds.
pub fn section_item_count(app: &App, section: FilterSection) -> usize {
    match section {
        FilterSection::Statuses => available_statuses(app).len().max(1),
        FilterSection::Priorities => 5,
        FilterSection::DueDate => 5,
        FilterSection::MeMode => 1,
        FilterSection::IncludeClosed => 1,
    }
}
