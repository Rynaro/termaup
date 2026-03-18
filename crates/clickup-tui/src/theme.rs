use ratatui::style::{Color, Modifier, Style};

/// Color theme for the TUI application.
pub struct Theme {
    /// Default foreground.
    pub fg: Color,
    /// Header / title foreground.
    pub header_fg: Color,
    /// Selected item background.
    pub selected_bg: Color,
    /// Selected item foreground.
    pub selected_fg: Color,
    /// Panel border color.
    pub border: Color,
    /// Breadcrumb bar foreground.
    pub breadcrumb_fg: Color,
    /// Error foreground.
    pub error_fg: Color,
    /// Error background.
    pub error_bg: Color,
    /// Success foreground.
    #[allow(dead_code)]
    pub success_fg: Color,
    /// Loading / spinner foreground.
    pub loading_fg: Color,
    /// Muted / secondary text.
    pub muted: Color,
    /// Search highlight color.
    pub search_fg: Color,
    /// Search bar background.
    pub search_bg: Color,
    /// Folder icon foreground.
    pub folder_fg: Color,
    /// List icon foreground.
    pub list_fg: Color,
}

impl Theme {
    /// Returns the default dark theme.
    pub fn default_dark() -> Self {
        Self {
            fg: Color::White,
            header_fg: Color::Cyan,
            selected_bg: Color::Rgb(50, 50, 80),
            selected_fg: Color::White,
            border: Color::Rgb(80, 80, 120),
            breadcrumb_fg: Color::Cyan,
            error_fg: Color::White,
            error_bg: Color::Red,
            success_fg: Color::Green,
            loading_fg: Color::Yellow,
            muted: Color::DarkGray,
            search_fg: Color::Yellow,
            search_bg: Color::Rgb(40, 40, 60),
            folder_fg: Color::Rgb(255, 200, 80),
            list_fg: Color::Rgb(100, 180, 255),
        }
    }

    /// Style for a selected list item.
    pub fn selected_style(&self) -> Style {
        Style::default()
            .fg(self.selected_fg)
            .bg(self.selected_bg)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for a normal (unselected) list item.
    pub fn normal_style(&self) -> Style {
        Style::default().fg(self.fg)
    }

    /// Style for panel borders.
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    /// Style for muted / secondary text.
    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted)
    }

    /// Style for header text.
    #[allow(dead_code)]
    pub fn header_style(&self) -> Style {
        Style::default()
            .fg(self.header_fg)
            .add_modifier(Modifier::BOLD)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::default_dark()
    }
}

/// Converts a hex colour string (e.g. "#d3d3d3") to a ratatui `Color`.
pub fn hex_to_color(hex: &str) -> Color {
    parse_hex(hex)
        .map(|(r, g, b)| Color::Rgb(r, g, b))
        .unwrap_or(Color::Gray)
}

/// Parses a hex colour string into (r, g, b).
pub fn parse_hex(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

/// Global theme instance.
pub static THEME: std::sync::LazyLock<Theme> = std::sync::LazyLock::new(Theme::default_dark);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_color() {
        assert_eq!(hex_to_color("#ff0000"), Color::Rgb(255, 0, 0));
        assert_eq!(hex_to_color("#00ff00"), Color::Rgb(0, 255, 0));
        assert_eq!(hex_to_color("invalid"), Color::Gray);
    }

    #[test]
    fn test_parse_hex() {
        assert_eq!(parse_hex("#d3d3d3"), Some((211, 211, 211)));
        assert_eq!(parse_hex("000000"), Some((0, 0, 0)));
        assert_eq!(parse_hex("short"), None);
    }

    #[test]
    fn test_theme_styles() {
        let theme = Theme::default_dark();
        let selected = theme.selected_style();
        assert_eq!(selected.fg, Some(Color::White));
    }
}
