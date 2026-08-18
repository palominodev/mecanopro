use crate::core::engine::TypingEngine;
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Gauge, Paragraph},
    Frame,
};

pub struct StatsBar;

impl StatsBar {
    pub fn render(f: &mut Frame, area: Rect, engine: &TypingEngine) {
        let metrics = engine.current_metrics();
        let is_golden_valid = metrics.accuracy >= engine.lesson.min_accuracy;

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(area);

        // 1. Warp Speed / CPM
        let cpm_span = vec![
            Line::from(vec![
                Span::styled("Velocidad: ", Style::default().fg(Theme::MUTED)),
                Span::styled(
                    format!("{:.0} CPM", metrics.cpm),
                    Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(format!("{:.1} WPM  ·  Meta: {:.0}", metrics.raw_wpm, engine.lesson.target_cpm), Style::default().fg(Theme::MUTED)),
            ]),
        ];
        f.render_widget(
            Paragraph::new(cpm_span)
                .block(Theme::retro_block("⚡ PROPULSIÓN", Theme::PRIMARY))
                .alignment(Alignment::Center),
            chunks[0],
        );

        // 2. Shields / Precision (Golden Rule)
        let accuracy_color = if is_golden_valid { Theme::SUCCESS } else { Theme::ERROR };
        let accuracy_spans = vec![
            Line::from(vec![
                Span::styled("Escudos: ", Style::default().fg(Theme::MUTED)),
                Span::styled(
                    format!("{:.1}%", metrics.accuracy),
                    Style::default().fg(accuracy_color).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    if is_golden_valid { "✓ Escudo Estable (≥96%)" } else { "⚠ Alerta: Requiere ≥96%" },
                    Style::default().fg(accuracy_color),
                ),
            ]),
        ];
        f.render_widget(
            Paragraph::new(accuracy_spans)
                .block(Theme::retro_block("🛡️ ESCUDOS", accuracy_color))
                .alignment(Alignment::Center),
            chunks[1],
        );

        // 3. Telemetry Time & Consistency
        let time_spans = vec![
            Line::from(vec![
                Span::styled("Tiempo: ", Style::default().fg(Theme::MUTED)),
                Span::styled(
                    format!("{:02}:{:02}", metrics.elapsed.as_secs() / 60, metrics.elapsed.as_secs() % 60),
                    Style::default().fg(Theme::TEXT).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(format!("Ritmo: {:.0}% sincro", metrics.consistency), Style::default().fg(Theme::NEBULA_PURPLE)),
            ]),
        ];
        f.render_widget(
            Paragraph::new(time_spans)
                .block(Theme::retro_block("⏱️ CRONOMETRÍA", Theme::NEBULA_PURPLE))
                .alignment(Alignment::Center),
            chunks[2],
        );

        // 4. Hyperdrive Jump Progress Gauge
        let progress_pct = (engine.progress_ratio() * 100.0).clamp(0.0, 100.0) as u16;
        let gauge = Gauge::default()
            .block(Theme::retro_block("🚀 HIPERSALTO", Theme::ACCENT))
            .gauge_style(
                Style::default()
                    .fg(Theme::ACCENT)
                    .bg(Theme::SURFACE)
                    .add_modifier(Modifier::BOLD),
            )
            .percent(progress_pct);
        f.render_widget(gauge, chunks[3]);
    }
}
