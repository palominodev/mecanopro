use ratatui::style::{Color, Modifier, Style};

pub struct Theme;

impl Theme {
    pub const PRIMARY: Color = Color::Rgb(0, 210, 255); // Vibrant Cyan
    pub const SECONDARY: Color = Color::Rgb(255, 170, 0); // Warm Amber
    pub const SUCCESS: Color = Color::Rgb(80, 250, 123); // Electric Green
    pub const ERROR: Color = Color::Rgb(255, 85, 85); // Bright Red
    pub const BG_DARK: Color = Color::Rgb(18, 20, 26);
    pub const SURFACE: Color = Color::Rgb(30, 34, 42);
    pub const MUTED: Color = Color::Rgb(108, 117, 125);
    pub const TEXT: Color = Color::Rgb(248, 248, 242);

    pub fn title_style() -> Style {
        Style::default()
            .fg(Self::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    pub fn target_char_style() -> Style {
        Style::default().fg(Self::MUTED)
    }

    pub fn correct_char_style() -> Style {
        Style::default().fg(Self::SUCCESS).add_modifier(Modifier::BOLD)
    }

    pub fn error_char_style() -> Style {
        Style::default()
            .fg(Self::ERROR)
            .bg(Color::Rgb(80, 20, 20))
            .add_modifier(Modifier::UNDERLINED | Modifier::BOLD)
    }

    pub fn cursor_char_style() -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(Self::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    pub fn golden_rule_style(is_valid: bool) -> Style {
        if is_valid {
            Style::default().fg(Self::SUCCESS).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Self::ERROR).add_modifier(Modifier::BOLD)
        }
    }
}
