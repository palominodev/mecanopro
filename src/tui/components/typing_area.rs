use crate::core::engine::TypingEngine;
use crate::tui::theme::Theme;
use ratatui::{
    layout::Alignment,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
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
            format!(" Escribiendo... [ Tecla muerta: {} ] ", dead)
        } else {
            format!(" Lección: {} ", engine.lesson.title)
        };

        Paragraph::new(Line::from(spans))
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Theme::PRIMARY)),
            )
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Left)
    }
}
