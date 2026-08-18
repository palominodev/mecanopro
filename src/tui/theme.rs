use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders};

pub struct Theme;

impl Theme {
    // ✦ Retro Space Synthwave Palette ✦
    pub const PRIMARY: Color = Color::Rgb(0, 240, 255);       // Neon Laser Cyan (Hyperdrive)
    pub const SECONDARY: Color = Color::Rgb(255, 42, 133);    // Synthwave Magenta (Supernova)
    pub const ACCENT: Color = Color::Rgb(255, 215, 0);        // Solar Flare Gold (Stars & Ranks)
    pub const SUCCESS: Color = Color::Rgb(57, 255, 20);       // Plasma Green (Shields 100% / Gate OK)
    pub const ERROR: Color = Color::Rgb(255, 51, 102);        // Warp Warning Red (Critical Error / Breached)
    pub const NEBULA_PURPLE: Color = Color::Rgb(187, 154, 247);// Cosmic Nebula Purple
    pub const BG_DARK: Color = Color::Rgb(11, 14, 20);        // Deep Cosmic Void
    pub const SURFACE: Color = Color::Rgb(22, 27, 34);        // Spaceship Hull Gray/Dark
    pub const MUTED: Color = Color::Rgb(108, 125, 147);       // Stardust Gray
    pub const TEXT: Color = Color::Rgb(240, 246, 252);        // Starlight Bright White

    pub fn title_style() -> Style {
        Style::default()
            .fg(Self::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    pub fn subtitle_style() -> Style {
        Style::default()
            .fg(Self::NEBULA_PURPLE)
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
            .bg(Color::Rgb(70, 10, 25))
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

    /// Retro container block with rounded borders and cyan/magenta glow
    pub fn retro_block(title: &str, border_color: Color) -> Block<'static> {
        Block::default()
            .title(format!(" {} ", title))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color))
    }

    /// Key shortcut badge styling [KEY] Action
    pub fn keybind_key_style() -> Style {
        Style::default()
            .fg(Self::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    pub fn keybind_desc_style() -> Style {
        Style::default().fg(Self::TEXT)
    }
}
