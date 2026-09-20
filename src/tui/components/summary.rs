use crate::core::model::{Lesson, SessionMetrics};
use crate::tui::theme::Theme;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
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
            Style::default()
                .fg(Theme::SUCCESS)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Theme::ERROR)
                .add_modifier(Modifier::BOLD)
        };

        let result_banner = if passed {
            " ★ ¡MISIÓN CUMPLIDA · HIPERSALTO EXITOSO! ★ "
        } else {
            " ⚠ ALERTA: ESCUDOS COMPROMETIDOS (<96% PRECISIÓN) ⚠ "
        };

        let mut lines = Vec::new();
        lines.push(Line::from(Span::styled(result_banner, title_style)));
        lines.push(Line::from(""));

        // Micro ASCII badge
        if passed {
            lines.push(Line::from(Span::styled(
                "   🚀 [ VECTOR DE SALTO ESTABLECIDO ] 🚀",
                Style::default().fg(Theme::PRIMARY),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "   /!\\ [ REGLA DE ORO: PRECISIÓN CRÍTICA ] /!\\",
                Style::default().fg(Theme::ERROR),
            )));
        }
        lines.push(Line::from(""));

        // Speed comparison
        lines.push(Line::from(vec![
            Span::styled("• Velocidad de Vuelo: ", Style::default().fg(Theme::MUTED)),
            Span::styled(
                format!("{:.0} CPM ({:.1} WPM)", metrics.cpm, metrics.raw_wpm),
                Style::default()
                    .fg(Theme::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  [Meta del Sector: {:.0} CPM]", lesson.target_cpm),
                Style::default().fg(Theme::MUTED),
            ),
        ]));

        // Accuracy comparison
        let acc_style = if metrics.accuracy >= lesson.min_accuracy {
            Style::default()
                .fg(Theme::SUCCESS)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Theme::ERROR)
                .add_modifier(Modifier::BOLD)
        };

        lines.push(Line::from(vec![
            Span::styled(
                "• Integridad de Escudos: ",
                Style::default().fg(Theme::MUTED),
            ),
            Span::styled(format!("{:.1}%", metrics.accuracy), acc_style),
            Span::styled(
                format!("  [Requerido: ≥{:.0}%]", lesson.min_accuracy),
                Style::default().fg(Theme::MUTED),
            ),
        ]));

        // Consistency & Errors
        lines.push(Line::from(vec![
            Span::styled("• Sincronía de Ritmo: ", Style::default().fg(Theme::MUTED)),
            Span::styled(
                format!("{:.0}%", metrics.consistency),
                Style::default()
                    .fg(Theme::NEBULA_PURPLE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "  (Turbulencias: {} de {} teclas)",
                    metrics.error_count, metrics.total_keystrokes
                ),
                Style::default().fg(Theme::MUTED),
            ),
        ]));

        // Dynamic Coaching Tips
        let tips =
            crate::core::feedback::FeedbackCoach::evaluate_session(metrics, lesson.min_accuracy);
        if !tips.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " 🤖 I.A. DE A BORDO (DIAGNÓSTICO): ",
                Style::default()
                    .fg(Theme::SECONDARY)
                    .add_modifier(Modifier::BOLD),
            )));
            for tip in tips {
                lines.push(Line::from(vec![
                    Span::styled(" ✦ ", Style::default().fg(Theme::ACCENT)),
                    Span::styled(tip, Style::default().fg(Theme::TEXT)),
                ]));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "[R] ",
                Style::default()
                    .fg(Theme::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Reintentar   ", Style::default().fg(Theme::TEXT)),
            Span::styled(
                "[S / ENTER] ",
                Style::default()
                    .fg(Theme::SUCCESS)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Siguiente Sector   ", Style::default().fg(Theme::TEXT)),
            Span::styled(
                "[ESC / M] ",
                Style::default()
                    .fg(Theme::MUTED)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Comando Central", Style::default().fg(Theme::TEXT)),
        ]));

        let modal_border_color = if passed { Theme::SUCCESS } else { Theme::ERROR };
        let modal_block = Theme::retro_block(
            &format!("✦ INFORME DE MISIÓN: {} ✦", lesson.title),
            modal_border_color,
        );

        let width = area.width.min(78);
        let height = area.height.min(18);
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
