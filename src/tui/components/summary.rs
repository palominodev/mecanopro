use crate::core::model::{Lesson, SessionMetrics};
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub struct SummaryModal;

impl SummaryModal {
    pub fn render(
        f: &mut Frame,
        area: Rect,
        lesson: &Lesson,
        metrics: &SessionMetrics,
        passed: bool,
    ) {
        let title_style = if passed {
            Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::ERROR).add_modifier(Modifier::BOLD)
        };

        let result_banner = if passed {
            " ★ ¡LECCIÓN COMPLETADA CON ÉXITO! ★ "
        } else {
            " ⚠ OBJETIVO NO ALCANZADO (REGLA DE ORO) ⚠ "
        };

        let mut lines = Vec::new();
        lines.push(Line::from(Span::styled(result_banner, title_style)));
        lines.push(Line::from(""));

        // Speed comparison
        lines.push(Line::from(vec![
            Span::styled("• Velocidad final: ", Style::default().fg(Theme::MUTED)),
            Span::styled(
                format!("{:.0} CPM ({:.1} WPM)", metrics.cpm, metrics.raw_wpm),
                Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  [Meta: {:.0} CPM]", lesson.target_cpm),
                Style::default().fg(Theme::MUTED),
            ),
        ]));

        // Accuracy comparison
        let acc_style = if metrics.accuracy >= lesson.min_accuracy {
            Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::ERROR).add_modifier(Modifier::BOLD)
        };

        lines.push(Line::from(vec![
            Span::styled("• Precisión: ", Style::default().fg(Theme::MUTED)),
            Span::styled(format!("{:.1}%", metrics.accuracy), acc_style),
            Span::styled(
                format!("  [Requerido: $\\ge${:.0}%]", lesson.min_accuracy),
                Style::default().fg(Theme::MUTED),
            ),
        ]));

        // Consistency & Errors
        lines.push(Line::from(vec![
            Span::styled("• Consistencia rítmica: ", Style::default().fg(Theme::MUTED)),
            Span::styled(
                format!("{:.0}%", metrics.consistency),
                Style::default().fg(Theme::TEXT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  (Errores: {} de {} teclas)", metrics.error_count, metrics.total_keystrokes),
                Style::default().fg(Theme::MUTED),
            ),
        ]));

        lines.push(Line::from(""));

        // Advice / Next steps
        if passed {
            lines.push(Line::from(Span::styled(
                "✓ Cumpliste con la Regla de Oro. Has desbloqueado el siguiente contenido.",
                Style::default().fg(Theme::SUCCESS),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "Recuerda: la velocidad es consecuencia de la precisión. No fuerces la rapidez sin asegurar el 96%.",
                Style::default().fg(Theme::SECONDARY),
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("[R] ", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled("Reintentar   ", Style::default().fg(Theme::TEXT)),
            Span::styled("[S / ENTER] ", Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)),
            Span::styled("Siguiente lección   ", Style::default().fg(Theme::TEXT)),
            Span::styled("[ESC / M] ", Style::default().fg(Theme::MUTED).add_modifier(Modifier::BOLD)),
            Span::styled("Menú principal", Style::default().fg(Theme::TEXT)),
        ]));

        let modal_block = Block::default()
            .title(format!(" Resumen de Sesión: {} ", lesson.title))
            .borders(Borders::ALL)
            .border_style(if passed {
                Style::default().fg(Theme::SUCCESS)
            } else {
                Style::default().fg(Theme::ERROR)
            });

        let width = area.width.min(75);
        let height = area.height.min(16);
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
