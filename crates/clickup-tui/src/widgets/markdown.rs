use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// Renders a markdown string into styled ratatui lines.
pub fn render_markdown(md: &str) -> Vec<Line<'static>> {
    let options = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES;
    let parser = Parser::new_ext(md, options);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut style_stack: Vec<Style> = vec![Style::default()];
    let mut in_code_block = false;
    let mut list_depth: usize = 0;
    let mut ordered_index: Option<u64> = None;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    flush_line(&mut lines, &mut current_spans);
                    let style = heading_style(level);
                    style_stack.push(style);
                }
                Tag::Paragraph => {
                    // Nothing special on start.
                }
                Tag::Strong => {
                    let base = current_style(&style_stack);
                    style_stack.push(base.add_modifier(Modifier::BOLD));
                }
                Tag::Emphasis => {
                    let base = current_style(&style_stack);
                    style_stack.push(base.add_modifier(Modifier::ITALIC));
                }
                Tag::CodeBlock(_) => {
                    flush_line(&mut lines, &mut current_spans);
                    lines.push(Line::from(Span::styled(
                        "  ┌───────────────────────",
                        Style::default().fg(Color::DarkGray),
                    )));
                    in_code_block = true;
                }
                Tag::Link { dest_url, .. } => {
                    let base = current_style(&style_stack);
                    style_stack.push(base.fg(Color::Blue).add_modifier(Modifier::UNDERLINED));
                    // Store the URL to append later.
                    let _ = dest_url;
                }
                Tag::List(start) => {
                    flush_line(&mut lines, &mut current_spans);
                    list_depth += 1;
                    ordered_index = start;
                }
                Tag::Item => {
                    flush_line(&mut lines, &mut current_spans);
                    let indent = "  ".repeat(list_depth);
                    let marker = if let Some(idx) = &mut ordered_index {
                        let m = format!("{indent}{idx}. ");
                        *idx += 1;
                        m
                    } else {
                        format!("{indent}• ")
                    };
                    current_spans.push(Span::styled(marker, Style::default().fg(Color::DarkGray)));
                }
                Tag::BlockQuote(_) => {
                    let base = current_style(&style_stack);
                    style_stack.push(base.fg(Color::DarkGray).add_modifier(Modifier::ITALIC));
                }
                _ => {}
            },

            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_) => {
                    flush_line(&mut lines, &mut current_spans);
                    lines.push(Line::from(""));
                    style_stack.pop();
                }
                TagEnd::Paragraph => {
                    flush_line(&mut lines, &mut current_spans);
                    lines.push(Line::from(""));
                }
                TagEnd::Strong | TagEnd::Emphasis => {
                    style_stack.pop();
                }
                TagEnd::CodeBlock => {
                    in_code_block = false;
                    lines.push(Line::from(Span::styled(
                        "  └───────────────────────",
                        Style::default().fg(Color::DarkGray),
                    )));
                    lines.push(Line::from(""));
                }
                TagEnd::Link => {
                    style_stack.pop();
                }
                TagEnd::List(_) => {
                    list_depth = list_depth.saturating_sub(1);
                    if list_depth == 0 {
                        ordered_index = None;
                    }
                    lines.push(Line::from(""));
                }
                TagEnd::Item => {
                    flush_line(&mut lines, &mut current_spans);
                }
                TagEnd::BlockQuote(_) => {
                    style_stack.pop();
                }
                _ => {}
            },

            Event::Text(text) => {
                let style = current_style(&style_stack);
                if in_code_block {
                    for line in text.lines() {
                        lines.push(Line::from(Span::styled(
                            format!("  │ {line}"),
                            Style::default().fg(Color::Green),
                        )));
                    }
                } else {
                    current_spans.push(Span::styled(text.to_string(), style));
                }
            }

            Event::Code(code) => {
                current_spans.push(Span::styled(
                    format!(" {code} "),
                    Style::default()
                        .fg(Color::Yellow)
                        .bg(Color::Rgb(40, 40, 40)),
                ));
            }

            Event::SoftBreak => {
                current_spans.push(Span::raw(" "));
            }

            Event::HardBreak => {
                flush_line(&mut lines, &mut current_spans);
            }

            Event::Rule => {
                flush_line(&mut lines, &mut current_spans);
                lines.push(Line::from(Span::styled(
                    "  ─".repeat(20),
                    Style::default().fg(Color::DarkGray),
                )));
                lines.push(Line::from(""));
            }

            _ => {}
        }
    }

    // Flush any remaining spans.
    flush_line(&mut lines, &mut current_spans);
    lines
}

fn current_style(stack: &[Style]) -> Style {
    stack.last().copied().unwrap_or_default()
}

fn flush_line(lines: &mut Vec<Line<'static>>, spans: &mut Vec<Span<'static>>) {
    if !spans.is_empty() {
        lines.push(Line::from(std::mem::take(spans)));
    }
}

fn heading_style(level: HeadingLevel) -> Style {
    match level {
        HeadingLevel::H1 => Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD),
        HeadingLevel::H2 => Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        _ => Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_heading() {
        let lines = render_markdown("# Hello World");
        assert!(!lines.is_empty());
        let first = &lines[0];
        assert!(!first.spans.is_empty());
        let text: String = first.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("Hello World"));
    }

    #[test]
    fn test_render_bold_italic() {
        let lines = render_markdown("**bold** and *italic*");
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_render_code_block() {
        let md = "```\nlet x = 1;\n```";
        let lines = render_markdown(md);
        let has_border = lines
            .iter()
            .any(|l| l.spans.iter().any(|s| s.content.contains('┌')));
        assert!(has_border);
    }

    #[test]
    fn test_render_list() {
        let md = "- item one\n- item two\n";
        let lines = render_markdown(md);
        let has_bullet = lines
            .iter()
            .any(|l| l.spans.iter().any(|s| s.content.contains('•')));
        assert!(has_bullet);
    }

    #[test]
    fn test_render_horizontal_rule() {
        let lines = render_markdown("---");
        let has_rule = lines
            .iter()
            .any(|l| l.spans.iter().any(|s| s.content.contains('─')));
        assert!(has_rule);
    }

    #[test]
    fn test_render_inline_code() {
        let lines = render_markdown("Use `foo()` here");
        let has_code = lines
            .iter()
            .any(|l| l.spans.iter().any(|s| s.content.contains("foo()")));
        assert!(has_code);
    }

    #[test]
    fn test_render_empty() {
        let lines = render_markdown("");
        assert!(lines.is_empty());
    }

    #[test]
    fn test_render_to_terminal_backend() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;
        use ratatui::widgets::Paragraph;

        let md = "# Title\n\nHello **world**\n";
        let lines = render_markdown(md);
        let paragraph = Paragraph::new(lines);

        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                frame.render_widget(paragraph, frame.area());
            })
            .unwrap();

        let buf = terminal.backend().buffer().clone();
        // The rendered buffer should contain our title text.
        let content: String = (0..buf.area.height)
            .flat_map(|y| {
                let buf = &buf;
                (0..buf.area.width).map(move |x| buf[(x, y)].symbol().to_string())
            })
            .collect();
        assert!(
            content.contains("Title"),
            "rendered buffer should contain heading text"
        );
        assert!(
            content.contains("world"),
            "rendered buffer should contain body text"
        );
    }

    #[test]
    fn test_render_complex_markdown() {
        let md = "\
# Heading 1

## Heading 2

Some text with **bold** and *italic*.

- Item one
- Item two

1. First
2. Second

`inline code`

```
code block
```

---

[link](https://example.com)
";
        let lines = render_markdown(md);
        // Should produce a reasonable number of lines.
        assert!(
            lines.len() > 10,
            "complex markdown should produce many lines, got {}",
            lines.len()
        );
    }
}
