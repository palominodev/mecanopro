use crate::core::dictation::DictationEngine;
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub struct DictationArea;

impl DictationArea {
    pub fn render(f: &mut Frame, area: Rect, engine: &DictationEngine) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Word sequence progress ribbon
                Constraint::Length(5), // Big masked word display
                Constraint::Length(3), // Audio & Interaction status banner
            ])
            .split(area);

        // 1. Word sequence progress ribbon
        let mut ribbon_spans = Vec::new();
        for (idx, word) in engine.words.iter().enumerate() {
            if idx < engine.current_word_index {
                ribbon_spans.push(Span::styled(
                    format!(" 🚀 {} ", word),
                    Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD),
                ));
            } else if idx == engine.current_word_index {
                ribbon_spans.push(Span::styled(
                    format!(" [ TRANSMISIÓN {}/{} ] ", idx + 1, engine.words.len()),
                    Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD),
                ));
            } else {
                ribbon_spans.push(Span::styled(
                    format!(" • Señal {} ", idx + 1),
                    Style::default().fg(Theme::MUTED),
                ));
            }
        }

        let ribbon = Paragraph::new(Line::from(ribbon_spans))
            .block(Theme::retro_block("✦ SECUENCIA DE TRANSMISIONES DE VOZ ✦", Theme::NEBULA_PURPLE))
            .alignment(Alignment::Center);
        f.render_widget(ribbon, chunks[0]);

        // 2. Big masked slots display
        let mut slot_spans = Vec::new();
        slot_spans.push(Span::raw("   "));

        let slots = engine.current_word_masked_slots();
        for (idx, &(ch, is_revealed)) in slots.iter().enumerate() {
            if is_revealed {
                slot_spans.push(Span::styled(
                    format!(" {} ", ch),
                    Style::default()
                        .fg(Theme::SUCCESS)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                ));
            } else if idx == engine.cursor_in_word {
                slot_spans.push(Span::styled(
                    " ▮ ",
                    Style::default()
                        .fg(Theme::PRIMARY)
                        .add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK),
                ));
            } else {
                slot_spans.push(Span::styled(
                    " ░ ",
                    Style::default()
                        .fg(Theme::MUTED)
                        .add_modifier(Modifier::BOLD),
                ));
            }
        }

        let word_title = if let Some(dead) = engine.pending_dead_key {
            format!("⚡ DECODIFICANDO AUDIO [ TECLA MUERTA: '{}' ]", dead)
        } else {
            "✦ DECODIFICADOR SUBESPACIAL (ESCRIBE LO QUE ESCUCHAS) ✦".to_string()
        };

        let word_widget = Paragraph::new(vec![Line::from(""), Line::from(slot_spans)])
            .block(Theme::retro_block(&word_title, Theme::PRIMARY))
            .alignment(Alignment::Center);
        f.render_widget(word_widget, chunks[1]);

        // 3. Audio & Status Banner
        let reaction_display = if let Some(last_reaction) = engine.word_reaction_times.last() {
            format!("Último reflejo: {:.0} ms", last_reaction.as_secs_f64() * 1000.0)
        } else {
            "Sintonizando canal auditivo...".to_string()
        };

        let status_spans = vec![
            Span::styled(" [📡 FRECUENCIA AUDIO] ", Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD)),
            Span::styled("Sintetizador: ", Style::default().fg(Theme::MUTED)),
            Span::styled(format!("{} ", engine.config.current_voice_name()), Style::default().fg(Theme::TEXT).add_modifier(Modifier::BOLD)),
            Span::styled("• ", Style::default().fg(Theme::MUTED)),
            Span::styled("Velocidad: ", Style::default().fg(Theme::MUTED)),
            Span::styled(format!("{:.1}x ", engine.config.speech_rate), Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled("• ", Style::default().fg(Theme::MUTED)),
            Span::styled(reaction_display, Style::default().fg(Theme::PRIMARY)),
            Span::styled(" • [TAB] ", Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD)),
            Span::styled(format!("Repetir ({}) ", engine.replay_count), Style::default().fg(Theme::TEXT)),
            Span::styled("• [F2 / Shift+TAB] ", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled("Cambiar voz", Style::default().fg(Theme::TEXT)),
        ];

        let status_bar = Paragraph::new(Line::from(status_spans))
            .block(Theme::retro_block("TELEMETRÍA DE AUDIO", Theme::MUTED))
            .alignment(Alignment::Center);
        f.render_widget(status_bar, chunks[2]);
    }
}
