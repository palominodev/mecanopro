use crate::tui::theme::Theme;
use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FingerZone {
    LeftPinky,
    LeftRing,
    LeftMiddle,
    LeftIndex,
    Thumb,
    RightIndex,
    RightMiddle,
    RightRing,
    RightPinky,
}

impl FingerZone {
    pub fn name(&self) -> &'static str {
        match self {
            Self::LeftPinky => "Mano Izquierda: Meñique",
            Self::LeftRing => "Mano Izquierda: Anular",
            Self::LeftMiddle => "Mano Izquierda: Medio",
            Self::LeftIndex => "Mano Izquierda: Índice",
            Self::Thumb => "Pulgar (Espacio)",
            Self::RightIndex => "Mano Derecha: Índice",
            Self::RightMiddle => "Mano Derecha: Medio",
            Self::RightRing => "Mano Derecha: Anular",
            Self::RightPinky => "Mano Derecha: Meñique",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::LeftPinky | Self::RightPinky => Color::Rgb(122, 162, 247), // Laser Blue
            Self::LeftRing | Self::RightRing => Color::Rgb(187, 154, 247),   // Nebula Purple
            Self::LeftMiddle | Self::RightMiddle => Color::Rgb(0, 240, 255), // Hyper Cyan
            Self::LeftIndex | Self::RightIndex => Color::Rgb(57, 255, 20),   // Plasma Green
            Self::Thumb => Color::Rgb(255, 215, 0),                          // Solar Gold
        }
    }
}

struct KeyCap {
    display: &'static str,
    char_match: Option<char>,
    finger: FingerZone,
    is_bump: bool,
}

impl KeyCap {
    const fn new(display: &'static str, c: char, finger: FingerZone) -> Self {
        Self {
            display,
            char_match: Some(c),
            finger,
            is_bump: false,
        }
    }

    const fn bump(display: &'static str, c: char, finger: FingerZone) -> Self {
        Self {
            display,
            char_match: Some(c),
            finger,
            is_bump: true,
        }
    }

    const fn modifier(display: &'static str, finger: FingerZone) -> Self {
        Self {
            display,
            char_match: None,
            finger,
            is_bump: false,
        }
    }
}

pub struct KeyboardVisualizer;

impl KeyboardVisualizer {
    fn row_numbers() -> Vec<KeyCap> {
        use FingerZone::*;
        vec![
            KeyCap::new(" º ", 'º', LeftPinky),
            KeyCap::new(" 1 ", '1', LeftPinky),
            KeyCap::new(" 2 ", '2', LeftRing),
            KeyCap::new(" 3 ", '3', LeftMiddle),
            KeyCap::new(" 4 ", '4', LeftIndex),
            KeyCap::new(" 5 ", '5', LeftIndex),
            KeyCap::new(" 6 ", '6', RightIndex),
            KeyCap::new(" 7 ", '7', RightIndex),
            KeyCap::new(" 8 ", '8', RightMiddle),
            KeyCap::new(" 9 ", '9', RightRing),
            KeyCap::new(" 0 ", '0', RightPinky),
            KeyCap::new(" ' ", '\'', RightPinky),
            KeyCap::new(" ¡ ", '¡', RightPinky),
            KeyCap::modifier("  BORRAR  ", RightPinky),
        ]
    }

    fn row_top() -> Vec<KeyCap> {
        use FingerZone::*;
        vec![
            KeyCap::modifier(" TAB  ", LeftPinky),
            KeyCap::new(" Q ", 'q', LeftPinky),
            KeyCap::new(" W ", 'w', LeftRing),
            KeyCap::new(" E ", 'e', LeftMiddle),
            KeyCap::new(" R ", 'r', LeftIndex),
            KeyCap::new(" T ", 't', LeftIndex),
            KeyCap::new(" Y ", 'y', RightIndex),
            KeyCap::new(" U ", 'u', RightIndex),
            KeyCap::new(" I ", 'i', RightMiddle),
            KeyCap::new(" O ", 'o', RightRing),
            KeyCap::new(" P ", 'p', RightPinky),
            KeyCap::new(" ` ", '`', RightPinky),
            KeyCap::new(" + ", '+', RightPinky),
            KeyCap::modifier("  ↵ ENTER ", RightPinky),
        ]
    }

    fn row_home() -> Vec<KeyCap> {
        use FingerZone::*;
        vec![
            KeyCap::modifier(" BLOQ ", LeftPinky),
            KeyCap::new(" A ", 'a', LeftPinky),
            KeyCap::new(" S ", 's', LeftRing),
            KeyCap::new(" D ", 'd', LeftMiddle),
            KeyCap::bump(" F̲ ", 'f', LeftIndex),
            KeyCap::new(" G ", 'g', LeftIndex),
            KeyCap::new(" H ", 'h', RightIndex),
            KeyCap::bump(" J̲ ", 'j', RightIndex),
            KeyCap::new(" K ", 'k', RightMiddle),
            KeyCap::new(" L ", 'l', RightRing),
            KeyCap::new(" Ñ ", 'ñ', RightPinky),
            KeyCap::new(" ´ ", '´', RightPinky),
            KeyCap::new(" Ç ", 'ç', RightPinky),
            KeyCap::modifier("          ", RightPinky),
        ]
    }

    fn row_bottom() -> Vec<KeyCap> {
        use FingerZone::*;
        vec![
            KeyCap::modifier(" SHIFT ", LeftPinky),
            KeyCap::new(" < ", '<', LeftPinky),
            KeyCap::new(" Z ", 'z', LeftPinky),
            KeyCap::new(" X ", 'x', LeftRing),
            KeyCap::new(" C ", 'c', LeftMiddle),
            KeyCap::new(" V ", 'v', LeftIndex),
            KeyCap::new(" B ", 'b', LeftIndex),
            KeyCap::new(" N ", 'n', RightIndex),
            KeyCap::new(" M ", 'm', RightIndex),
            KeyCap::new(" , ", ',', RightMiddle),
            KeyCap::new(" . ", '.', RightRing),
            KeyCap::new(" - ", '-', RightPinky),
            KeyCap::modifier("   SHIFT   ", RightPinky),
        ]
    }

    pub fn render(target_char: Option<char>) -> Paragraph<'static> {
        let active_char = target_char.map(|c| c.to_lowercase().next().unwrap_or(c));
        let active_finger = Self::finger_for_char(target_char);

        let rows = vec![
            Self::row_numbers(),
            Self::row_top(),
            Self::row_home(),
            Self::row_bottom(),
        ];

        let mut lines = Vec::new();

        for row in rows {
            let mut spans = Vec::new();
            for key in row {
                let is_active = match (active_char, key.char_match) {
                    (Some(c), Some(k)) if c == k => true,
                    (Some('á'), Some('a' | '´')) => true,
                    (Some('é'), Some('e' | '´')) => true,
                    (Some('í'), Some('i' | '´')) => true,
                    (Some('ó'), Some('o' | '´')) => true,
                    (Some('ú'), Some('u' | '´')) => true,
                    (Some('ü'), Some('u' | '¨')) => true,
                    _ => false,
                };

                let style = if is_active {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Theme::SECONDARY)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                } else if key.is_bump {
                    // Tactile guide bump key (F & J)
                    Style::default()
                        .fg(key.finger.color())
                        .bg(Theme::SURFACE)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                } else if key.char_match.is_none() {
                    // Utility key (Tab, Shift, Enter)
                    Style::default()
                        .fg(Theme::MUTED)
                        .bg(Color::Rgb(24, 26, 32))
                } else {
                    // Standard key colored with its ergonomic finger zone
                    Style::default()
                        .fg(Theme::TEXT)
                        .bg(Theme::SURFACE)
                };

                spans.push(Span::styled(format!("[{}]", key.display), style));
                spans.push(Span::raw(" "));
            }
            lines.push(Line::from(spans));
        }

        // Spacebar Row
        let is_space_active = active_char == Some(' ');
        let space_style = if is_space_active {
            Style::default()
                .fg(Color::Black)
                .bg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Theme::TEXT)
                .bg(Theme::SURFACE)
        };

        let space_row = vec![
            Span::styled("[ CTRL ]", Style::default().fg(Theme::MUTED).bg(Color::Rgb(24, 26, 32))),
            Span::raw(" "),
            Span::styled("[ ALT ]", Style::default().fg(Theme::MUTED).bg(Color::Rgb(24, 26, 32))),
            Span::raw("  "),
            Span::styled("[                    ESPACIO                    ]", space_style),
            Span::raw("  "),
            Span::styled("[ ALT GR ]", Style::default().fg(Theme::MUTED).bg(Color::Rgb(24, 26, 32))),
            Span::raw(" "),
            Span::styled("[ CTRL ]", Style::default().fg(Theme::MUTED).bg(Color::Rgb(24, 26, 32))),
        ];
        lines.push(Line::from(space_row));
        lines.push(Line::from(""));

        // Finger Guidance Banner
        let finger_spans = vec![
            Span::styled("  Dedo Activo: ", Style::default().fg(Theme::MUTED)),
            Span::styled(
                format!(" {} ", active_finger.name()),
                Style::default()
                    .fg(Color::Black)
                    .bg(active_finger.color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("   •   Guía Fila Base: ", Style::default().fg(Theme::MUTED)),
            Span::styled("ASDF", Style::default().fg(FingerZone::LeftIndex.color()).add_modifier(Modifier::BOLD)),
            Span::styled(" (Izq) / ", Style::default().fg(Theme::MUTED)),
            Span::styled("JKLÑ", Style::default().fg(FingerZone::RightIndex.color()).add_modifier(Modifier::BOLD)),
            Span::styled(" (Der)", Style::default().fg(Theme::MUTED)),
        ];
        lines.push(Line::from(finger_spans));

        Paragraph::new(lines)
            .block(
                Theme::retro_block(
                    "✦ CABINA DE MANDO: TECLADO ESPAÑOL ISO (GUÍA ERGONÓMICA) ✦",
                    Theme::PRIMARY,
                )
            )
            .alignment(Alignment::Center)
    }

    pub fn finger_for_char(c: Option<char>) -> FingerZone {
        use FingerZone::*;
        match c.map(|ch| ch.to_lowercase().next().unwrap_or(ch)) {
            Some('1' | 'q' | 'a' | 'z' | 'º' | '<') => LeftPinky,
            Some('2' | 'w' | 's' | 'x') => LeftRing,
            Some('3' | 'e' | 'd' | 'c' | 'é') => LeftMiddle,
            Some('4' | '5' | 'r' | 't' | 'f' | 'g' | 'v' | 'b') => LeftIndex,
            Some(' ') => Thumb,
            Some('6' | '7' | 'y' | 'u' | 'h' | 'j' | 'n' | 'm' | 'ú' | 'ü') => RightIndex,
            Some('8' | 'i' | 'k' | ',' | 'í') => RightMiddle,
            Some('9' | 'o' | 'l' | '.' | 'ó') => RightRing,
            Some('0' | '\'' | '¡' | 'p' | '+' | 'ñ' | '´' | 'ç' | '-' | '¿' | '?' | '!') => RightPinky,
            Some('á') => RightPinky, // Dead key accent first
            _ => Thumb,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finger_zones_mapping() {
        assert_eq!(KeyboardVisualizer::finger_for_char(Some('a')), FingerZone::LeftPinky);
        assert_eq!(KeyboardVisualizer::finger_for_char(Some('f')), FingerZone::LeftIndex);
        assert_eq!(KeyboardVisualizer::finger_for_char(Some('j')), FingerZone::RightIndex);
        assert_eq!(KeyboardVisualizer::finger_for_char(Some('ñ')), FingerZone::RightPinky);
        assert_eq!(KeyboardVisualizer::finger_for_char(Some(' ')), FingerZone::Thumb);
    }
}
