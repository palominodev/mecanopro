use crate::tui::theme::Theme;
use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub struct KeyboardVisualizer;

impl KeyboardVisualizer {
    pub fn render(target_char: Option<char>) -> Paragraph<'static> {
        let active_char = target_char.map(|c| c.to_lowercase().next().unwrap_or(c));

        let rows = vec![
            vec!['º', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '\'', '¡'],
            vec!['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '`', '+'],
            vec!['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', 'ñ', '´', 'ç'],
            vec!['<', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '-'],
        ];

        let mut lines = Vec::new();

        for row in rows {
            let mut spans = Vec::new();
            spans.push(Span::raw("  "));
            for key in row {
                let is_active = match (active_char, key) {
                    (Some(c), k) if c == k => true,
                    (Some('á'), 'a') | (Some('á'), '´') => true,
                    (Some('é'), 'e') | (Some('é'), '´') => true,
                    (Some('í'), 'i') | (Some('í'), '´') => true,
                    (Some('ó'), 'o') | (Some('ó'), '´') => true,
                    (Some('ú'), 'u') | (Some('ú'), '´') => true,
                    (Some('ü'), 'u') | (Some('ü'), '¨') => true,
                    _ => false,
                };

                let style = if is_active {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Theme::PRIMARY)
                        .add_modifier(Modifier::BOLD)
                } else if key == 'f' || key == 'j' {
                    // Guide bumps
                    Style::default()
                        .fg(Theme::SECONDARY)
                        .add_modifier(Modifier::UNDERLINED | Modifier::BOLD)
                } else {
                    Style::default().fg(Theme::TEXT)
                };

                spans.push(Span::styled(format!("[ {} ]", key.to_uppercase()), style));
                spans.push(Span::raw(" "));
            }
            lines.push(Line::from(spans));
        }

        // Spacebar row
        let is_space = active_char == Some(' ');
        let space_style = if is_space {
            Style::default()
                .fg(Color::Black)
                .bg(Theme::PRIMARY)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::TEXT)
        };

        lines.push(Line::from(vec![
            Span::raw("          "),
            Span::styled("[                ESPACIO                ]", space_style),
        ]));

        // Finger guidance hint
        let finger_hint = Self::finger_hint_for_char(target_char);
        lines.push(Line::from(vec![
            Span::styled(" Dedo recomendado: ", Style::default().fg(Theme::MUTED)),
            Span::styled(finger_hint, Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD)),
        ]));

        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Teclado Español (ISO) ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Theme::PRIMARY)),
            )
            .alignment(Alignment::Center)
    }

    pub fn finger_hint_for_char(c: Option<char>) -> &'static str {
        match c.map(|ch| ch.to_lowercase().next().unwrap_or(ch)) {
            Some('1' | 'q' | 'a' | 'z' | 'º' | '<') => "Mano Izquierda: Meñique",
            Some('2' | 'w' | 's' | 'x') => "Mano Izquierda: Anular",
            Some('3' | 'e' | 'd' | 'c' | 'é') => "Mano Izquierda: Medio",
            Some('4' | '5' | 'r' | 't' | 'f' | 'g' | 'v' | 'b') => "Mano Izquierda: Índice",
            Some(' ') => "Cualquier Pulgar",
            Some('6' | '7' | 'y' | 'u' | 'h' | 'j' | 'n' | 'm' | 'ú' | 'ü') => "Mano Derecha: Índice",
            Some('8' | 'i' | 'k' | ',' | 'í') => "Mano Derecha: Medio",
            Some('9' | 'o' | 'l' | '.' | 'ó') => "Mano Derecha: Anular",
            Some('0' | '\'' | '¡' | 'p' | '+' | 'ñ' | '´' | 'ç' | '-' | '¿' | '?' | '!') => "Mano Derecha: Meñique",
            Some('á') => "Meñique derecho (´) + Meñique izq (A)",
            _ => "Posición neutral en fila guía (ASDF - JKLÑ)",
        }
    }
}
