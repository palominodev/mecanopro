use crate::core::engine::TypingEngine;
use crate::tui::theme::Theme;
use ratatui::{
    layout::Alignment,
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

pub struct TypingArea;

impl TypingArea {
    pub fn render(engine: &TypingEngine) -> Paragraph<'static> {
        let mut spans = Vec::new();

        for (idx, &ch) in engine.target_chars.iter().enumerate() {
            if idx < engine.cursor {
                let display_ch = if ch == ' ' { '·' } else { ch };
                spans.push(Span::styled(
                    display_ch.to_string(),
                    Theme::correct_char_style(),
                ));
            } else if idx == engine.cursor {
                let display_ch = if ch == ' ' { ' ' } else { ch };
                spans.push(Span::styled(
                    display_ch.to_string(),
                    Theme::cursor_char_style(),
                ));
            } else {
                let display_ch = if ch == ' ' { ' ' } else { ch };
                spans.push(Span::styled(
                    display_ch.to_string(),
                    Theme::target_char_style(),
                ));
            }
        }

        let title = if let Some(dead) = engine.pending_dead_key {
            format!("⚡ TRANSMISIÓN ENTRADA · [ TECLA MUERTA ACTIVA: '{}' ]", dead)
        } else {
            format!("✦ TRANSMISIÓN DE VUELO: {} ✦", engine.lesson.title)
        };

        Paragraph::new(Line::from(spans))
            .block(Theme::retro_block(&title, Theme::PRIMARY))
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Left)
    }
}
