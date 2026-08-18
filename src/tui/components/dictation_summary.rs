use crate::core::dictation::DictationMetrics;
use crate::core::feedback::FeedbackCoach;
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};

pub struct DictationSummaryModal;

impl DictationSummaryModal {
    pub fn render(f: &mut Frame, area: Rect, metrics: &DictationMetrics, replay_count: usize) {
        let is_passed = metrics.accuracy >= 96.0;

        let title_style = if is_passed {
            Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::ERROR).add_modifier(Modifier::BOLD)
        };

        let result_banner = if is_passed {
            " ★ ¡MISIÓN DE DICTADO COMPLETADA CON ÉXITO! ★ "
        } else {
            " ⚠ ALERTA DE DICTADO: MEJORAR PRECISIÓN (<96%) ⚠ "
        };

        let mut lines = Vec::new();
        lines.push(Line::from(Span::styled(result_banner, title_style)));
        lines.push(Line::from(""));

        // Reaction Time (Primary Dictation Diagnostic)
        let reaction_eval = if metrics.avg_reaction_time_ms < 350.0 {
            "Reflejos de piloto de élite (Extraordinario)"
        } else if metrics.avg_reaction_time_ms < 600.0 {
            "Buen tiempo de respuesta neural"
        } else {
            "Respuesta lenta, se recomienda calibrar en simulador"
        };

        lines.push(Line::from(vec![
            Span::styled("• Reflejo Auditivo-Motor: ", Style::default().fg(Theme::MUTED)),
            Span::styled(
                format!("{:.0} ms (promedio)", metrics.avg_reaction_time_ms),
                Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  [Mín: {:.0} ms | Máx: {:.0} ms]", metrics.min_reaction_time_ms, metrics.max_reaction_time_ms), Style::default().fg(Theme::MUTED)),
        ]));

        lines.push(Line::from(vec![
            Span::styled("  Diagnóstico Telemetría: ", Style::default().fg(Theme::MUTED)),
            Span::styled(reaction_eval, Style::default().fg(Theme::SECONDARY)),
        ]));

        // Speed comparison (Active typing CPM)
        lines.push(Line::from(vec![
            Span::styled("• Velocidad de Propulsión: ", Style::default().fg(Theme::MUTED)),
            Span::styled(
                format!("{:.0} CPM ({:.1} WPM)", metrics.cpm, metrics.raw_wpm),
                Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  [Tiempo activo: {:.1}s]", metrics.active_typing_duration.as_secs_f64()),
                Style::default().fg(Theme::MUTED),
            ),
        ]));

        // Accuracy
        let acc_style = if is_passed {
            Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::ERROR).add_modifier(Modifier::BOLD)
        };

        lines.push(Line::from(vec![
            Span::styled("• Integridad de Escudos: ", Style::default().fg(Theme::MUTED)),
            Span::styled(format!("{:.1}%", metrics.accuracy), acc_style),
            Span::styled(
                format!("  (Turbulencias: {} en {} palabras)", metrics.error_count, metrics.total_words),
                Style::default().fg(Theme::MUTED),
            ),
        ]));

        // Consistency
        lines.push(Line::from(vec![
            Span::styled("• Sincronía Rítmica: ", Style::default().fg(Theme::MUTED)),
            Span::styled(
                format!("{:.0}%", metrics.consistency),
                Style::default().fg(Theme::NEBULA_PURPLE).add_modifier(Modifier::BOLD),
            ),
        ]));

        // Dynamic Feedback & Coaching Tips
        let tips = FeedbackCoach::evaluate_dictation(metrics, replay_count);
        if !tips.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(" 🤖 I.A. DE A BORDO (CONSEJOS DE VUELO): ", Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD))));
            for tip in tips {
                lines.push(Line::from(vec![
                    Span::styled(" ✦ ", Style::default().fg(Theme::ACCENT)),
                    Span::styled(tip, Style::default().fg(Theme::TEXT)),
                ]));
            }
        }

        lines.push(Line::from(""));

        // Footer Actions
        lines.push(Line::from(vec![
            Span::styled("[R] ", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled("Reintentar   ", Style::default().fg(Theme::TEXT)),
            Span::styled("[N / ENTER] ", Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)),
            Span::styled("Nueva Transmisión   ", Style::default().fg(Theme::TEXT)),
            Span::styled("[ESC / M] ", Style::default().fg(Theme::MUTED).add_modifier(Modifier::BOLD)),
            Span::styled("Comando Central", Style::default().fg(Theme::TEXT)),
        ]));

        let modal_border_color = if is_passed { Theme::SUCCESS } else { Theme::ERROR };
        let modal_block = Theme::retro_block(
            "✦ INFORME DE TELEMETRÍA AUDITIVA ✦",
            modal_border_color,
        );

        let width = area.width.min(88);
        let height = area.height.min(22);
        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;
        let modal_area = Rect::new(x, y, width, height);

        f.render_widget(Clear, modal_area);
        f.render_widget(
            Paragraph::new(lines)
                .block(modal_block)
                .alignment(Alignment::Center),
            modal_area,
        );
    }
}
