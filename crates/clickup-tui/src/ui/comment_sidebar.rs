use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};

use crate::app::{App, CommentInputMode};
use crate::theme::THEME;

/// Renders the comment sidebar panel.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let title = format!(" Comments ({}) ", app.comments.len());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(THEME.border_style());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 2 || inner.width < 10 {
        return;
    }

    // Reserve space for the input area at the bottom when composing.
    let (comment_area, input_area) = if app.comment_input_mode != CommentInputMode::Browse {
        // Estimate visual lines: each logical line may wrap based on the width.
        // Use inner.width - 2 to account for the leading space and cursor char.
        let usable_width = (inner.width as usize).saturating_sub(2).max(1);
        let visual_text_lines: usize = app
            .comment_input_text
            .split('\n')
            .map(|line| {
                let len = line.len();
                if len == 0 {
                    1
                } else {
                    len.div_ceil(usable_width)
                }
            })
            .sum::<usize>()
            .max(1);
        // prompt (1) + visual text lines
        let desired = (visual_text_lines as u16) + 1;
        // Cap at 40% of inner height or 10 rows, whichever is smaller.
        let max_input = (inner.height * 2 / 5).clamp(3, 10);
        let input_height = desired.min(max_input).min(inner.height.saturating_sub(2));
        let split = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Min(1),
                ratatui::layout::Constraint::Length(input_height),
            ])
            .split(inner);
        (split[0], Some(split[1]))
    } else {
        (inner, None)
    };

    // Build the flattened comment lines with selection highlighting.
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut flat_index: usize = 0;

    if app.comments.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No comments yet",
            THEME.muted_style(),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Press n to add one",
            THEME.muted_style(),
        )));
    }

    for comment in &app.comments {
        let is_selected = flat_index == app.selected_comment_index;
        let date_str = format_relative_date(&comment.date);

        let author_style = if is_selected {
            Style::default()
                .fg(Color::Cyan)
                .bg(THEME.selected_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        };
        let date_style = if is_selected {
            Style::default().fg(Color::DarkGray).bg(THEME.selected_bg)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let bg_style = if is_selected {
            Style::default().bg(THEME.selected_bg)
        } else {
            Style::default()
        };

        lines.push(Line::from(vec![
            Span::styled("  ", bg_style),
            Span::styled(
                comment
                    .user
                    .as_ref()
                    .map(|u| u.username.clone())
                    .unwrap_or_else(|| "Unknown".to_string()),
                author_style,
            ),
            Span::styled(" · ", date_style),
            Span::styled(date_str, date_style),
        ]));

        // Comment text lines
        for text_line in comment.comment_text.lines() {
            lines.push(Line::from(vec![
                Span::styled("    ", bg_style),
                Span::styled(text_line.to_string(), bg_style.fg(THEME.fg)),
            ]));
        }

        // Thread indicator
        let is_expanded = app.expanded_comments.contains(&comment.id);
        if comment.reply_count > 0 {
            let indicator = if is_expanded {
                format!("    ▼ {} replies", comment.reply_count)
            } else {
                format!("    ▶ {} replies", comment.reply_count)
            };
            lines.push(Line::from(Span::styled(
                indicator,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::DIM),
            )));
        }

        flat_index += 1;

        // Render expanded replies
        if is_expanded {
            if let Some(replies) = app.comment_replies.get(&comment.id) {
                for (i, reply) in replies.iter().enumerate() {
                    let is_reply_selected = flat_index == app.selected_comment_index;
                    let is_last = i == replies.len() - 1;
                    let connector = if is_last { "└─" } else { "├─" };
                    let reply_date = format_relative_date(&reply.date);

                    let r_author_style = if is_reply_selected {
                        Style::default()
                            .fg(Color::Cyan)
                            .bg(THEME.selected_bg)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    };
                    let r_date_style = if is_reply_selected {
                        Style::default().fg(Color::DarkGray).bg(THEME.selected_bg)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };
                    let r_bg_style = if is_reply_selected {
                        Style::default().bg(THEME.selected_bg)
                    } else {
                        Style::default()
                    };

                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("    {connector} "),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(
                            reply
                                .user
                                .as_ref()
                                .map(|u| u.username.clone())
                                .unwrap_or_else(|| "Unknown".to_string()),
                            r_author_style,
                        ),
                        Span::styled(" · ", r_date_style),
                        Span::styled(reply_date, r_date_style),
                    ]));

                    let indent = if is_last { "       " } else { "    │  " };
                    for text_line in reply.comment_text.lines() {
                        lines.push(Line::from(vec![
                            Span::styled(indent.to_string(), Style::default().fg(Color::DarkGray)),
                            Span::styled(text_line.to_string(), r_bg_style.fg(THEME.fg)),
                        ]));
                    }

                    flat_index += 1;
                }
            } else {
                // Replies are being loaded
                lines.push(Line::from(Span::styled(
                    "    ⠋ Loading replies…",
                    THEME.muted_style(),
                )));
            }
        }

        lines.push(Line::from("")); // spacing between comments
    }

    // Render comment list with scroll
    let scroll = app.comment_scroll_offset;
    let content = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    frame.render_widget(content, comment_area);

    // Render input area if composing
    if let Some(input_rect) = input_area {
        render_input_area(app, frame, input_rect);
    }
}

/// Renders the comment input area at the bottom of the sidebar.
fn render_input_area(app: &App, frame: &mut Frame, area: Rect) {
    let prompt = match &app.comment_input_mode {
        CommentInputMode::NewComment => "New comment:".to_string(),
        CommentInputMode::EditComment => "Edit comment:".to_string(),
        CommentInputMode::Reply => {
            if let Some(ref target_id) = app.reply_target_id {
                let username = app
                    .comments
                    .iter()
                    .find(|c| c.id == *target_id)
                    .and_then(|c| c.user.as_ref().map(|u| u.username.as_str()))
                    .or_else(|| {
                        app.comment_replies
                            .values()
                            .flatten()
                            .find(|c| c.id == *target_id)
                            .and_then(|c| c.user.as_ref().map(|u| u.username.as_str()))
                    })
                    .unwrap_or("comment");
                format!("Reply to {username}:")
            } else {
                "Reply:".to_string()
            }
        }
        CommentInputMode::Browse => return,
    };

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut input_lines = vec![Line::from(Span::styled(
        format!(" {prompt} (Ctrl+D to send)"),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ))];

    // Render each line of the multi-line input text.
    let text = &app.comment_input_text;
    if text.is_empty() {
        // Show cursor on empty input.
        input_lines.push(Line::from(vec![
            Span::raw(" "),
            Span::styled("█", Style::default().fg(THEME.fg)),
        ]));
    } else {
        let lines_iter: Vec<&str> = text.split('\n').collect();
        let total = lines_iter.len();
        for (i, line) in lines_iter.into_iter().enumerate() {
            let is_last = i == total - 1;
            if is_last {
                // Show cursor at end of last line.
                input_lines.push(Line::from(vec![
                    Span::raw(" "),
                    Span::styled(line.to_string(), Style::default().fg(THEME.fg)),
                    Span::styled("█", Style::default().fg(THEME.fg)),
                ]));
            } else {
                input_lines.push(Line::from(vec![
                    Span::raw(" "),
                    Span::styled(line.to_string(), Style::default().fg(THEME.fg)),
                ]));
            }
        }
    }

    // Scroll the input if it exceeds the available height.
    // Account for visual line wrapping — a single logical line may span
    // multiple visual rows when Wrap is enabled.
    let area_width = inner.width.max(1) as usize;
    let visible_rows = inner.height as usize;
    let total_visual_lines: usize = input_lines
        .iter()
        .map(|line| {
            let char_width: usize = line.spans.iter().map(|s| s.content.len()).sum();
            if char_width == 0 {
                1
            } else {
                char_width.div_ceil(area_width)
            }
        })
        .sum();
    let scroll_offset = if total_visual_lines > visible_rows {
        (total_visual_lines - visible_rows) as u16
    } else {
        0
    };

    let input_widget = Paragraph::new(input_lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll_offset, 0));
    frame.render_widget(input_widget, inner);
}

/// Formats a millisecond timestamp as a relative or short date string.
fn format_relative_date(timestamp_ms: &str) -> String {
    let ms: i64 = match timestamp_ms.parse() {
        Ok(v) => v,
        Err(_) => return timestamp_ms.to_string(),
    };
    let Some(dt) = chrono::DateTime::from_timestamp_millis(ms) else {
        return timestamp_ms.to_string();
    };
    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(dt);

    if diff.num_minutes() < 1 {
        "just now".to_string()
    } else if diff.num_hours() < 1 {
        let mins = diff.num_minutes();
        format!("{mins}m ago")
    } else if diff.num_hours() < 24 {
        let hours = diff.num_hours();
        format!("{hours}h ago")
    } else if diff.num_days() < 7 {
        let days = diff.num_days();
        format!("{days}d ago")
    } else {
        dt.format("%b %d, %Y").to_string()
    }
}

/// Renders the delete confirmation overlay on top of the sidebar.
pub fn render_delete_confirm(frame: &mut Frame, area: Rect) {
    let popup_width = 36_u16.min(area.width.saturating_sub(4));
    let popup_height = 5_u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    // Clear the area behind the popup.
    let clear = ratatui::widgets::Clear;
    frame.render_widget(clear, popup_area);

    let block = Block::default()
        .title(" Confirm Delete ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Red));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let lines = vec![
        Line::from(Span::styled(
            "Delete this comment?",
            Style::default().fg(THEME.fg).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "y",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(":confirm  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "n",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(":cancel", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let paragraph = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(paragraph, inner);
}
