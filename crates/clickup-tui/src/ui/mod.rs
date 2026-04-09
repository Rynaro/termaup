pub mod comment_sidebar;
pub mod filter_panel;
pub mod help;
pub mod layout;
pub mod list_view;
pub mod search;
pub mod space_content;
pub mod space_list;
pub mod task_detail;
pub mod vision_board;
pub mod vision_sections;
pub mod workspace_select;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::app::{App, CommentInputMode, Screen, ViewMode};
use crate::theme::THEME;

/// Minimum terminal columns for usable display.
const MIN_COLS: u16 = 60;
/// Minimum terminal rows for usable display.
const MIN_ROWS: u16 = 15;

/// Renders the full UI frame based on the current app state.
pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();

    // Terminal too small guard.
    if area.width < MIN_COLS || area.height < MIN_ROWS {
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "Terminal too small",
                Style::default()
                    .fg(THEME.error_fg)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!(
                    "Need at least {MIN_COLS}×{MIN_ROWS}, \
                     got {}×{}",
                    area.width, area.height
                ),
                Style::default().fg(THEME.muted),
            )),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(msg, area);
        return;
    }

    // Determine if we need a search bar row.
    let has_search = app.filter_active;
    let bottom_rows = if has_search { 2 } else { 1 };

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),           // top bar
            Constraint::Min(1),              // main
            Constraint::Length(bottom_rows), // bottom
        ])
        .split(area);

    // --- Top bar: breadcrumb + last updated ---
    render_top_bar(app, frame, outer[0]);

    // --- Main area ---
    render_screen(app, frame, outer[1]);

    // --- Bottom bar(s) ---
    if has_search {
        let bottom = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // search bar
                Constraint::Length(1), // key hints
            ])
            .split(outer[2]);
        search::render_search_bar(app, frame, bottom[0]);
        render_bottom_bar(app, frame, bottom[1]);
    } else {
        render_bottom_bar(app, frame, outer[2]);
    }

    // --- Help overlay ---
    if app.show_help {
        help::render(app, frame);
    }

    // --- Filter panel overlay ---
    if app.filter_panel_open {
        filter_panel::render(app, frame);
    }
}

fn render_top_bar(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    let breadcrumb_text = app.breadcrumb.join(" › ");
    let mut left_spans = vec![Span::styled(
        format!(" {breadcrumb_text} "),
        Style::default()
            .fg(THEME.breadcrumb_fg)
            .add_modifier(Modifier::BOLD),
    )];

    if app.loading {
        left_spans.push(Span::styled(
            format!(" {} ", app.spinner_char()),
            Style::default().fg(THEME.loading_fg),
        ));
    }

    // Filter indicators.
    if app.screen == Screen::TaskList && app.task_filters.is_active() {
        let mut badges = Vec::new();
        if app.task_filters.me_mode {
            badges.push("👤 Me");
        }
        if !app.task_filters.statuses.is_empty() {
            badges.push("◉ Status");
        }
        if !app.task_filters.priorities.is_empty() {
            badges.push("⚑ Priority");
        }
        if app.task_filters.due_date_filter != clickup_api::filter_config::DueDateFilter::All {
            badges.push("📅 Due");
        }
        if app.task_filters.include_closed {
            badges.push("✓ Closed");
        }
        if !badges.is_empty() {
            left_spans.push(Span::styled(
                format!(" [{}] ", badges.join(" · ")),
                Style::default().fg(THEME.search_fg),
            ));
        }
    }

    // Right side: last updated + help hint.
    let updated = app.last_updated_label();
    let mut right_parts = Vec::new();
    if !updated.is_empty() {
        right_parts.push(Span::styled(
            format!("↻ {updated}  "),
            Style::default().fg(THEME.muted),
        ));
    }
    right_parts.push(Span::styled("? help ", Style::default().fg(THEME.muted)));

    // Calculate padding.
    let left_len: usize = left_spans.iter().map(|s| s.width()).sum();
    let right_len: usize = right_parts.iter().map(|s| s.width()).sum();
    let pad = (area.width as usize).saturating_sub(left_len + right_len);

    left_spans.push(Span::raw(" ".repeat(pad)));
    left_spans.extend(right_parts);

    frame.render_widget(Paragraph::new(Line::from(left_spans)), area);
}

fn render_bottom_bar(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    if let Some(err) = &app.error_message {
        let bar = Paragraph::new(Line::from(Span::styled(
            format!(" ✗ {err} "),
            Style::default().fg(THEME.error_fg).bg(THEME.error_bg),
        )));
        frame.render_widget(bar, area);
        return;
    }

    let bottom_spans = build_key_hints(app);
    frame.render_widget(Paragraph::new(Line::from(bottom_spans)), area);
}

/// Dispatches to screen-specific renderers.
fn render_screen(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    if app.loading && screen_data_empty(app) {
        let spinner = Paragraph::new(format!("  {} Loading…", app.spinner_char()))
            .alignment(Alignment::Left)
            .style(Style::default().fg(THEME.loading_fg))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(THEME.border_style()),
            );
        frame.render_widget(spinner, area);
        return;
    }

    match app.screen {
        Screen::Loading => {
            let loading = Paragraph::new(format!("  {} Connecting…", app.spinner_char()))
                .style(Style::default().fg(THEME.loading_fg));
            frame.render_widget(loading, area);
        }
        Screen::WorkspaceSelect => {
            workspace_select::render(app, frame, area);
        }
        Screen::SpaceList => {
            space_list::render(app, frame, area);
        }
        Screen::SpaceContent => {
            space_content::render(app, frame, area);
        }
        Screen::TaskList => match app.view_mode {
            ViewMode::List => list_view::render(app, frame, area),
            ViewMode::VisionSections => vision_sections::render(app, frame, area),
            ViewMode::VisionBoard => vision_board::render(app, frame, area),
        },
        Screen::TaskDetail => {
            if app.comment_sidebar_open {
                let split = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                    .split(area);
                task_detail::render(app, frame, split[0]);
                comment_sidebar::render(app, frame, split[1]);
                // Render the delete confirmation overlay on top of the sidebar.
                if app.delete_confirm_target.is_some() {
                    comment_sidebar::render_delete_confirm(frame, split[1]);
                }
            } else {
                task_detail::render(app, frame, area);
            }
        }
    }
}

/// Returns true if there's no data to display for the current screen.
fn screen_data_empty(app: &App) -> bool {
    match app.screen {
        Screen::Loading => true,
        Screen::WorkspaceSelect => app.workspaces.is_empty(),
        Screen::SpaceList => app.spaces.is_empty(),
        Screen::SpaceContent => app.space_content.is_empty() && app.folders.is_empty(),
        Screen::TaskList => app.tasks.is_empty(),
        Screen::TaskDetail => app.current_task.is_none(),
    }
}

/// Builds key hint spans for the bottom bar.
fn build_key_hints(app: &App) -> Vec<Span<'static>> {
    let hints = key_hints(app);
    hints
        .iter()
        .enumerate()
        .flat_map(|(i, (key, desc))| {
            let mut spans = vec![
                Span::styled(
                    format!(" {key} "),
                    Style::default().fg(THEME.selected_fg).bg(THEME.muted),
                ),
                Span::styled(format!(" {desc} "), Style::default().fg(THEME.muted)),
            ];
            if i < hints.len() - 1 {
                spans.push(Span::raw(" "));
            }
            spans
        })
        .collect()
}

/// Returns key hint pairs for the current screen.
fn key_hints(app: &App) -> Vec<(&'static str, &'static str)> {
    let mut hints = vec![];

    match app.screen {
        Screen::Loading => {}
        Screen::WorkspaceSelect => {
            hints.push(("↑/k", "Up"));
            hints.push(("↓/j", "Down"));
            hints.push(("Enter", "Select"));
            hints.push(("/", "Search"));
            hints.push(("r", "Refresh"));
        }
        Screen::SpaceList => {
            hints.push(("↑/k", "Up"));
            hints.push(("↓/j", "Down"));
            hints.push(("Enter", "Select"));
            hints.push(("Esc", "Back"));
            hints.push(("/", "Search"));
            hints.push(("r", "Refresh"));
        }
        Screen::SpaceContent => {
            hints.push(("↑/k", "Up"));
            hints.push(("↓/j", "Down"));
            hints.push(("Enter", "Open"));
            hints.push(("Esc", "Back"));
            hints.push(("/", "Search"));
            hints.push(("r", "Refresh"));
        }
        Screen::TaskList => {
            hints.push(("↑/k", "Up"));
            hints.push(("↓/j", "Down"));
            if matches!(
                app.view_mode,
                ViewMode::VisionSections | ViewMode::VisionBoard
            ) {
                hints.push(("Tab", "Next group"));
            }
            hints.push(("Enter", "View"));
            hints.push(("Esc", "Back"));
            hints.push(("v", "View mode"));
            hints.push(("/", "Search"));
            hints.push(("f", "Filters"));
            hints.push(("m", "Me Mode"));
            hints.push(("r", "Refresh"));
        }
        Screen::TaskDetail => {
            if app.delete_confirm_target.is_some() {
                hints.push(("y", "Confirm delete"));
                hints.push(("n", "Cancel"));
            } else if app.comment_input_mode != CommentInputMode::Browse {
                hints.push(("Enter", "Send"));
                hints.push(("Alt+Enter/^N", "Newline"));
                hints.push(("Esc", "Cancel"));
            } else if app.comment_sidebar_open {
                hints.push(("↑/k", "Nav comments"));
                hints.push(("↓/j", "Nav comments"));
                hints.push(("n", "New comment"));
                hints.push(("e", "Edit"));
                hints.push(("d", "Delete"));
                hints.push(("c", "Close comments"));
                hints.push(("Esc", "Back"));
            } else {
                hints.push(("↑/k", "Scroll up"));
                hints.push(("↓/j", "Scroll down"));
                hints.push(("c", "Comments"));
                hints.push(("Esc", "Back"));
            }
        }
    }

    // Suppress global shortcuts during compose — those keys feed into the text box.
    if app.comment_input_mode == CommentInputMode::Browse {
        hints.push(("?", "Help"));
        hints.push(("q", "Quit"));
    }
    hints
}
