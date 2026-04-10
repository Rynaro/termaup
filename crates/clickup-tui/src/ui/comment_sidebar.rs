use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};

use clickup_api::models::CommentContentItem;

use crate::app::{App, CommentInputMode};
use crate::theme::THEME;

/// Builds a list of ratatui [`Span`]s from a comment's structured content
/// array, styling @mention tags as cyan+bold.
///
/// Falls back to rendering `plain_text` if `content` is empty.
fn comment_body_spans<'a>(
    content: &'a [CommentContentItem],
    plain_text: &'a str,
    base_style: Style,
) -> Vec<Span<'a>> {
    if content.is_empty() {
        return plain_text
            .split('\n')
            .flat_map(|l| [Span::styled(l.to_string(), base_style), Span::raw("\n")])
            .collect();
    }
    content
        .iter()
        .map(|item| match item {
            CommentContentItem::Tag { user, text, .. } => {
                let label = text
                    .as_deref()
                    .or_else(|| user.username.as_deref().map(|_| "@"))
                    .unwrap_or("@mention");
                Span::styled(
                    label.to_string(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            }
            CommentContentItem::Text { text, .. } => Span::styled(text.clone(), base_style),
            CommentContentItem::Unknown(_) => Span::raw(""),
        })
        .collect()
}

/// Renders a set of comment body lines (handles multi-line, newlines from rich
/// content) with the given indent and base style.
fn render_comment_body_lines<'a>(
    content: &'a [CommentContentItem],
    plain_text: &'a str,
    indent: &'a str,
    bg_style: Style,
) -> Vec<Line<'a>> {
    let mut out: Vec<Line<'a>> = Vec::new();

    if content.is_empty() {
        for line_text in plain_text.lines() {
            let mut line_spans = vec![Span::styled(indent.to_string(), bg_style)];
            // Highlight @username tokens in input text.
            line_spans.extend(highlight_at_mentions_in_text(line_text, bg_style));
            out.push(Line::from(line_spans));
        }
        return out;
    }

    // Collect all spans, then split on newline spans.
    let mut current_line: Vec<Span<'a>> = vec![Span::styled(indent.to_string(), bg_style)];
    for item in content {
        match item {
            CommentContentItem::Tag { user, text, .. } => {
                let label = if let Some(t) = text.as_deref() {
                    t.to_string()
                } else if let Some(uname) = user.username.as_deref() {
                    format!("@{uname}")
                } else {
                    "@mention".to_string()
                };
                current_line.push(Span::styled(
                    label,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ));
            }
            CommentContentItem::Text { text, .. } => {
                // Split on newlines.
                let parts: Vec<&str> = text.split('\n').collect();
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        out.push(Line::from(current_line.drain(..).collect::<Vec<_>>()));
                        current_line = vec![Span::styled(indent.to_string(), bg_style)];
                    }
                    if !part.is_empty() {
                        current_line.push(Span::styled((*part).to_string(), bg_style));
                    }
                }
            }
            CommentContentItem::Unknown(_) => {}
        }
    }
    if current_line.len() > 1 {
        // Only emit if there's content beyond the indent.
        out.push(Line::from(current_line));
    }
    out
}

/// Highlights `@username` tokens in raw text with cyan+bold styling.
///
/// Used for received comment bodies where no mention map is available;
/// uses an alphanumeric-only token scanner.
fn highlight_at_mentions_in_text(text: &str, base_style: Style) -> Vec<Span<'_>> {
    let mut spans: Vec<Span<'_>> = Vec::new();
    let mut last = 0;
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes[i] == b'@' {
            let at_boundary = i == 0 || (bytes[i - 1] as char).is_whitespace();
            if at_boundary {
                // Collect token.
                let token_start = i + 1;
                let mut j = token_start;
                while j < len
                    && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b'-')
                {
                    j += 1;
                }
                if j > token_start {
                    if i > last {
                        spans.push(Span::styled(&text[last..i], base_style));
                    }
                    spans.push(Span::styled(
                        &text[i..j],
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ));
                    last = j;
                    i = j;
                    continue;
                }
            }
        }
        i += 1;
    }
    if last < len {
        spans.push(Span::styled(&text[last..], base_style));
    }
    spans
}

/// Highlights `@mention` tokens in compose input text using `mention_map` for
/// longest-match (handles multi-word display names), with alphanumeric scanner
/// as fallback for manually typed tokens.
fn highlight_compose_mentions<'a>(
    text: &'a str,
    base_style: Style,
    mention_map: &std::collections::HashMap<String, i64>,
) -> Vec<Span<'a>> {
    let mut spans: Vec<Span<'a>> = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0usize;
    let mut last_byte = 0usize;

    // Sort keys longest-first for greedy match.
    let mut map_keys: Vec<&String> = mention_map.keys().collect();
    map_keys.sort_by(|a, b| b.chars().count().cmp(&a.chars().count()));

    // Track byte offset alongside char index.
    let char_byte_offsets: Vec<usize> = {
        let mut offsets = vec![0usize; len + 1];
        let mut byte = 0;
        for (idx, c) in chars.iter().enumerate() {
            offsets[idx] = byte;
            byte += c.len_utf8();
        }
        offsets[len] = byte;
        offsets
    };

    let mention_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);

    while i < len {
        if chars[i] == '@' {
            let at_boundary = i == 0 || chars[i - 1].is_whitespace();
            if at_boundary {
                // Try mention_map longest-match.
                let rest: String = chars[i + 1..].iter().collect();
                let mut matched_chars: Option<usize> = None;
                for key in &map_keys {
                    if rest.starts_with(key.as_str()) {
                        let end = i + 1 + key.chars().count();
                        if end >= len || chars[end].is_whitespace() {
                            matched_chars = Some(key.chars().count());
                            break;
                        }
                    }
                }
                if let Some(token_chars) = matched_chars {
                    let end = i + 1 + token_chars;
                    let start_byte = char_byte_offsets[i];
                    let end_byte = char_byte_offsets[end];
                    if start_byte > last_byte {
                        spans.push(Span::styled(&text[last_byte..start_byte], base_style));
                    }
                    spans.push(Span::styled(&text[start_byte..end_byte], mention_style));
                    last_byte = end_byte;
                    i = end;
                    continue;
                }

                // Fallback: alphanumeric scanner.
                let token_start = i + 1;
                let mut j = token_start;
                while j < len
                    && (chars[j].is_alphanumeric()
                        || chars[j] == '_'
                        || chars[j] == '-'
                        || chars[j] == '.')
                {
                    j += 1;
                }
                if j > token_start {
                    let start_byte = char_byte_offsets[i];
                    let end_byte = char_byte_offsets[j];
                    if start_byte > last_byte {
                        spans.push(Span::styled(&text[last_byte..start_byte], base_style));
                    }
                    spans.push(Span::styled(&text[start_byte..end_byte], mention_style));
                    last_byte = end_byte;
                    i = j;
                    continue;
                }
            }
        }
        i += 1;
    }

    let total_bytes = char_byte_offsets[len];
    if last_byte < total_bytes {
        spans.push(Span::styled(&text[last_byte..], base_style));
    }
    spans
}

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
    let mut lines: Vec<Line<'_>> = Vec::new();
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
        for body_line in render_comment_body_lines(
            &comment.comment,
            &comment.comment_text,
            "    ",
            bg_style,
        ) {
            lines.push(body_line);
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
                    for body_line in render_comment_body_lines(
                        &reply.comment,
                        &reply.comment_text,
                        indent,
                        r_bg_style,
                    ) {
                        lines.push(body_line);
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
        // Render the mention picker overlay on top when active.
        if app.mention_picker_active {
            render_mention_picker(app, frame, input_rect);
        }
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
        format!(" {prompt} (Enter to send, Alt+Enter or Ctrl+N for newline)"),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ))];

    // Render each line of the multi-line input text, with @mention highlighting.
    let text = &app.comment_input_text;
    let base_fg = Style::default().fg(THEME.fg);
    if text.is_empty() {
        // Show cursor on empty input.
        input_lines.push(Line::from(vec![
            Span::raw(" "),
            Span::styled("█", base_fg),
        ]));
    } else {
        let lines_iter: Vec<&str> = text.split('\n').collect();
        let total = lines_iter.len();
        for (i, line) in lines_iter.into_iter().enumerate() {
            let is_last = i == total - 1;
            let mut spans = vec![Span::raw(" ")];
            spans.extend(highlight_compose_mentions(line, base_fg, &app.comment_mention_map));
            if is_last {
                spans.push(Span::styled("█", base_fg));
            }
            input_lines.push(Line::from(spans));
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

/// Renders the floating mention picker overlay above the input area.
///
/// Shows up to 5 filtered workspace members. The selected row is highlighted.
/// An empty member list shows a "No members found" message.
pub fn render_mention_picker(app: &App, frame: &mut Frame, input_area: Rect) {
    let members = app.filtered_members();
    let visible_count = members.len().min(5);
    // Height: border top + border bottom + rows (at least 1 for "no members")
    let picker_height = (visible_count.max(1) as u16) + 2;
    let picker_width = input_area.width.min(40).max(20);

    // Position: just above the input area, right-aligned within the sidebar.
    let y = input_area.y.saturating_sub(picker_height);
    let x = input_area.x + input_area.width.saturating_sub(picker_width);
    let picker_area = Rect {
        x,
        y,
        width: picker_width,
        height: picker_height,
    };

    // Clear the area so the picker renders cleanly over whatever is behind it.
    frame.render_widget(Clear, picker_area);

    let filter = &app.mention_picker_filter;
    let title = if filter.is_empty() {
        " @mention ".to_string()
    } else {
        format!(" @{filter} ")
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(picker_area);
    frame.render_widget(block, picker_area);

    if members.is_empty() {
        let msg = Paragraph::new(Span::styled(
            " No members found",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(msg, inner);
        return;
    }

    // Compute scroll so selected row is always visible (up to 5 rows).
    let max_visible: usize = inner.height as usize;
    let selected = app.mention_picker_selected;
    let scroll_start = if selected >= max_visible {
        selected - max_visible + 1
    } else {
        0
    };

    let layout_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            (0..max_visible.min(members.len()))
                .map(|_| Constraint::Length(1))
                .collect::<Vec<_>>(),
        )
        .split(inner);

    for (row_idx, chunk) in layout_chunks.iter().enumerate() {
        let member_idx = scroll_start + row_idx;
        if let Some(member) = members.get(member_idx) {
            let is_selected = member_idx == selected;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.fg)
            };
            let label = format!(
                " {} ",
                member.user.username
            );
            frame.render_widget(Paragraph::new(Span::styled(label, style)), *chunk);
        }
    }
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
