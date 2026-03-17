# Markdown Rendering Specification — clickup-tui

Convert `pulldown-cmark` events to `Vec<ratatui::text::Line>` with styled `Span`s.

## Element Mapping

| Markdown element | ratatui rendering |
|-----------------|-------------------|
| `# H1` | Bold, Blue foreground |
| `## H2` | Bold, Cyan foreground |
| `### H3` | Bold, White foreground |
| `#### H4+` | Bold, Gray foreground |
| `**bold**` | `Style::new().bold()` modifier |
| `*italic*` | `Style::new().italic()` modifier |
| `` `code span` `` | Gray background (`Color::DarkGray`), White foreground |
| Code block (```) | Bordered `Block` with `DarkGray` background, render each line inside |
| `- item` / `* item` | Indent with `"  • "` prefix |
| `1. item` | Indent with `"  1. "` prefix (increment counter) |
| `[text](url)` | Blue foreground, Underlined modifier |
| `---` (horizontal rule) | Full-width line of `"─"` characters |
| `> blockquote` | `"│ "` prefix in Gray, italic text |
| Paragraph break | Empty `Line` |

## Implementation Notes

- Use `pulldown-cmark::Parser::new_ext(md, Options::all())` for full markdown support
- Track state with a style stack: push modifiers on `Start` events, pop on `End` events
- Accumulate `Span`s for the current line, flush to a `Line` on newlines
- Handle nested formatting (e.g., bold inside a list item) by composing style modifiers
- Code blocks should be collected as a group and rendered as a bordered block
- Limit line width to the available terminal width minus padding
- Return `Vec<Line<'static>>` — clone all strings to avoid lifetime issues
