use crate::tui::app::{App, CurrentView};
use crate::tui::components::{KeyboardVisualizer, StatsBar, SummaryModal, TypingArea};
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    match app.current_view {
        CurrentView::MainMenu => render_main_menu(f, app),
        CurrentView::Practice => render_practice(f, app),
        CurrentView::Summary => {
            render_practice(f, app);
            if let (Some(engine), Some(metrics)) = (&app.current_engine, &app.last_session_metrics) {
                SummaryModal::render(f, f.area(), &engine.lesson, metrics, app.last_session_passed);
            }
        }
        CurrentView::Stats => render_stats(f, app),
    }
}

fn render_main_menu(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(12),
            Constraint::Length(3),
        ])
        .split(f.area());

    // 1. Header
    let header_lines = vec![
        Line::from(vec![
            Span::styled(" MECANOPRO ", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled("— Tutor de Mecanografía en Español (Meta: 150 CPM / 30 WPM)", Style::default().fg(Theme::TEXT)),
        ]),
        Line::from(vec![
            Span::styled(" Regla de Oro: ", Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD)),
            Span::styled("Precisión obligatoria $\\ge$ 96% para desbloquear niveles.", Style::default().fg(Theme::MUTED)),
        ]),
    ];
    let header = Paragraph::new(header_lines)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Theme::PRIMARY)))
        .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    // 2. Body: Left = Lessons list, Right = User Stats & Tier Status
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(chunks[1]);

    let lessons = app.available_lessons();
    let mut list_items = Vec::new();

    for (idx, lesson) in lessons.iter().enumerate() {
        let is_selected = idx == app.selected_lesson_index;
        let score = app.user_progress.completed_lessons.get(&lesson.id);

        let status_badge = match score {
            Some(s) if s.passed => Span::styled(" [✓ PASADO] ", Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)),
            Some(_) => Span::styled(" [⚠ REPETIR] ", Style::default().fg(Theme::ERROR)),
            None => Span::styled(" [  NUEVO  ] ", Style::default().fg(Theme::MUTED)),
        };

        let best_stat = if let Some(s) = score {
            format!("({:.0} CPM | {:.1}%)", s.cpm, s.accuracy)
        } else {
            format!("(Meta: {:.0} CPM)", lesson.target_cpm)
        };

        let prefix = if is_selected { " ▶ " } else { "   " };
        let item_style = if is_selected {
            Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::TEXT)
        };

        list_items.push(ListItem::new(Line::from(vec![
            Span::styled(prefix, item_style),
            status_badge,
            Span::styled(format!("{:<30} ", lesson.title), item_style),
            Span::styled(best_stat, Style::default().fg(Theme::MUTED)),
        ])));
    }

    let lessons_list = List::new(list_items)
        .block(Block::default().title(" Plan de Estudios y Niveles ").borders(Borders::ALL));
    f.render_widget(lessons_list, body_chunks[0]);

    // Sidebar: User Overview
    let total_mins = app.user_progress.total_practice_seconds / 60;
    let completed_count = app.user_progress.completed_lessons.values().filter(|v| v.passed).count();

    let sidebar_lines = vec![
        Line::from(vec![
            Span::styled("Nivel Actual: ", Style::default().fg(Theme::MUTED)),
            Span::styled(app.user_progress.unlocked_tier.name(), Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Lecciones aprobadas: ", Style::default().fg(Theme::MUTED)),
            Span::styled(format!("{}/{}", completed_count, lessons.len()), Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Tiempo total de práctica: ", Style::default().fg(Theme::MUTED)),
            Span::styled(format!("{} min", total_mins), Style::default().fg(Theme::TEXT)),
        ]),
        Line::from(""),
        Line::from(Span::styled("Teclas con mayor atención:", Style::default().fg(Theme::SECONDARY))),
    ];

    let mut sidebar_all_lines = sidebar_lines;
    let mut weak_keys: Vec<_> = app.user_progress.key_stats.iter().collect();
    weak_keys.sort_by(|a, b| b.1.error_rate().partial_cmp(&a.1.error_rate()).unwrap_or(std::cmp::Ordering::Equal));

    for (ch, stat) in weak_keys.into_iter().take(4) {
        if stat.errors > 0 {
            sidebar_all_lines.push(Line::from(vec![
                Span::styled(format!(" • Tecla '{}': ", ch), Style::default().fg(Theme::ERROR)),
                Span::styled(format!("{:.1}% errores ({} fallos)", stat.error_rate(), stat.errors), Style::default().fg(Theme::MUTED)),
            ]));
        }
    }

    let sidebar = Paragraph::new(sidebar_all_lines)
        .block(Block::default().title(" Perfil y Métricas ").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(sidebar, body_chunks[1]);

    // 3. Footer Keybinds
    let footer_spans = vec![
        Span::styled("[↑/↓ o j/k] ", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
        Span::styled("Navegar  ", Style::default().fg(Theme::TEXT)),
        Span::styled("[ENTER] ", Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)),
        Span::styled("Iniciar lección  ", Style::default().fg(Theme::TEXT)),
        Span::styled("[D] ", Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD)),
        Span::styled("Drill Adaptativo  ", Style::default().fg(Theme::TEXT)),
        Span::styled("[E] ", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
        Span::styled("Estadísticas  ", Style::default().fg(Theme::TEXT)),
        Span::styled("[Q] ", Style::default().fg(Theme::MUTED).add_modifier(Modifier::BOLD)),
        Span::styled("Salir", Style::default().fg(Theme::TEXT)),
    ];
    let footer = Paragraph::new(Line::from(footer_spans))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}

fn render_practice(f: &mut Frame, app: &App) {
    if let Some(engine) = &app.current_engine {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Length(6),
                Constraint::Min(9),
                Constraint::Length(3),
            ])
            .split(f.area());

        // 1. Stats Bar (Speed, accuracy, time, progress)
        StatsBar::render(f, chunks[0], engine);

        // 2. Typing Area
        let typing_widget = TypingArea::render(engine);
        f.render_widget(typing_widget, chunks[1]);

        // 3. Spanish ISO Keyboard Visualizer
        let target_char = engine.current_expected_char();
        let keyboard_widget = KeyboardVisualizer::render(target_char);
        f.render_widget(keyboard_widget, chunks[2]);

        // 4. Practice Footer
        let footer_spans = vec![
            Span::styled("[ESC] ", Style::default().fg(Theme::MUTED).add_modifier(Modifier::BOLD)),
            Span::styled("Volver al menú   ", Style::default().fg(Theme::TEXT)),
            Span::styled("[TAB] ", Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD)),
            Span::styled("Reiniciar lección   ", Style::default().fg(Theme::TEXT)),
            Span::styled("Fila Guía: Mantén dedos en ASDF - JKLÑ sin mirar", Style::default().fg(Theme::MUTED)),
        ];
        let footer = Paragraph::new(Line::from(footer_spans))
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(footer, chunks[3]);
    }
}

fn render_stats(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(f.area());

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(" MAPA DE RENDIMIENTO Y PRECISIÓN POR TECLA ", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD))));
    lines.push(Line::from(""));

    let mut stats: Vec<_> = app.user_progress.key_stats.iter().collect();
    stats.sort_by_key(|b| std::cmp::Reverse(b.1.attempts));

    if stats.is_empty() {
        lines.push(Line::from(Span::styled("Aún no hay datos de sesiones registradas. ¡Completa lecciones para generar tu mapa de calor!", Style::default().fg(Theme::MUTED))));
    } else {
        lines.push(Line::from(vec![
            Span::styled(format!("{:<8} {:<12} {:<12} {:<15} {:<15}", "Tecla", "Intentos", "Errores", "% Error", "Latencia Med."), Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(Span::styled("─".repeat(65), Style::default().fg(Theme::MUTED))));

        for (ch, stat) in stats.into_iter().take(12) {
            let error_color = if stat.error_rate() > 4.0 { Theme::ERROR } else { Theme::SUCCESS };
            let display_char = if *ch == ' ' { "ESPACIO".to_string() } else { format!("'{}'", ch) };

            lines.push(Line::from(vec![
                Span::styled(format!("{:<8} ", display_char), Style::default().fg(Theme::TEXT)),
                Span::styled(format!("{:<12} ", stat.attempts), Style::default().fg(Theme::MUTED)),
                Span::styled(format!("{:<12} ", stat.errors), Style::default().fg(Theme::MUTED)),
                Span::styled(format!("{:<14.1}% ", stat.error_rate()), Style::default().fg(error_color).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:.0} ms", stat.avg_latency_ms()), Style::default().fg(Theme::TEXT)),
            ]));
        }
    }

    let stats_widget = Paragraph::new(lines)
        .block(Block::default().title(" Estadísticas Históricas ").borders(Borders::ALL))
        .alignment(Alignment::Left);
    f.render_widget(stats_widget, chunks[0]);

    let footer_spans = vec![
        Span::styled("[D] ", Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD)),
        Span::styled("Iniciar Drill de Teclas Débiles   ", Style::default().fg(Theme::TEXT)),
        Span::styled("[ESC / ENTER / Q] ", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
        Span::styled("Volver al Menú Principal", Style::default().fg(Theme::TEXT)),
    ];
    let footer = Paragraph::new(Line::from(footer_spans))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[1]);
}
