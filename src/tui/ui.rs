use crate::core::model::Tier;
use crate::core::Curriculum;
use crate::tui::app::{App, CurrentView};
use crate::tui::ascii::AsciiArt;
use crate::tui::components::{
    DictationArea, DictationSummaryModal, KeyboardVisualizer, StatsBar, SummaryModal, TypingArea,
};
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
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
        CurrentView::Dictation => render_dictation(f, app),
        CurrentView::DictationSummary => {
            render_dictation(f, app);
            if let Some(metrics) = &app.last_dictation_metrics {
                let replay_count = app
                    .current_dictation
                    .as_ref()
                    .map(|e| e.replay_count)
                    .unwrap_or(0);
                DictationSummaryModal::render(f, f.area(), metrics, replay_count);
            }
        }
    }
}

fn render_main_menu(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),  // Retro ASCII Header
            Constraint::Min(12),   // Body (Sectors & Telemetry)
            Constraint::Length(3),  // Retro Footer
        ])
        .split(f.area());

    // 1. Retro ASCII Header Banner
    let header_lines = vec![
        Line::from(Span::styled(
            AsciiArt::LOGO_LINES[0],
            Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            AsciiArt::LOGO_LINES[1],
            Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            AsciiArt::SUBTITLE,
            Style::default().fg(Theme::NEBULA_PURPLE).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("✦ REGLA DE ORO ESTELAR: ", Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled("Precisión obligatoria ≥ 96% para desbloquear sectores de navegación estelar", Style::default().fg(Theme::TEXT)),
        ]),
    ];
    let header = Paragraph::new(header_lines)
        .block(Theme::retro_block("COMANDO CENTRAL :: MECANOPRO", Theme::PRIMARY))
        .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    // 2. Body: Left = Sectors List, Right = Pilot Telemetry & Stats
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(chunks[1]);

    let lessons = app.available_lessons();
    let sections = Curriculum::all_sections();
    let visible_capacity = (body_chunks[0].height.saturating_sub(2)).max(1) as usize;
    let selected = app.selected_lesson_index;

    // One display row per lesson, plus a non-selectable header row whenever
    // the curriculum section changes. Selection stays lesson-based.
    enum MenuRow<'a> {
        Header(&'a str),
        Lesson(usize),
    }

    let mut display_rows: Vec<MenuRow<'_>> = Vec::with_capacity(lessons.len() + sections.len());
    let mut current_section: Option<&str> = None;
    for (idx, lesson) in lessons.iter().enumerate() {
        if current_section != Some(lesson.section_id.as_str()) {
            let title = sections
                .iter()
                .find(|s| s.id == lesson.section_id)
                .map(|s| s.title.as_str())
                .unwrap_or(lesson.section_id.as_str());
            display_rows.push(MenuRow::Header(title));
            current_section = Some(lesson.section_id.as_str());
        }
        display_rows.push(MenuRow::Lesson(idx));
    }

    let selected_display_idx = display_rows
        .iter()
        .position(|row| matches!(row, MenuRow::Lesson(idx) if *idx == selected))
        .unwrap_or(0);

    // Viewport window calculated over DISPLAY rows (lessons + section headers)
    let start_idx = if selected_display_idx >= visible_capacity {
        selected_display_idx + 1 - visible_capacity
    } else {
        0
    };

    let mut list_items = Vec::new();

    for row in display_rows.iter().skip(start_idx).take(visible_capacity) {
        match row {
            MenuRow::Header(title) => {
                list_items.push(ListItem::new(Line::from(Span::styled(
                    format!("  ── SECCIÓN: {} ──", title),
                    Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD),
                ))));
            }
            MenuRow::Lesson(idx) => {
                let lesson = &lessons[*idx];
                let is_selected = *idx == selected;
                let score = app.user_progress.completed_lessons.get(&lesson.id);

                let status_badge = match score {
                    Some(s) if s.passed => Span::styled(" [🚀 DESPEJADO] ", Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)),
                    Some(_) => Span::styled(" [⚠ ALERTA]    ", Style::default().fg(Theme::ERROR).add_modifier(Modifier::BOLD)),
                    None => Span::styled(" [✦ INEXPLORADO]", Style::default().fg(Theme::MUTED)),
                };

                let best_stat = if let Some(s) = score {
                    format!("({:.0} CPM · {:.1}%)", s.cpm, s.accuracy)
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
                    Span::styled(format!(" {:<44} ", lesson.title), item_style),
                    Span::styled(best_stat, Style::default().fg(Theme::ACCENT)),
                ])));
            }
        }
    }

    let menu_title = format!(
        "✦ SECTORES [{}/{}] · SCROLL [PgUp/PgDn/g/G/Ctrl+d/u] ✦",
        selected + 1,
        lessons.len()
    );

    let lessons_list = List::new(list_items)
        .block(Theme::retro_block(&menu_title, Theme::PRIMARY));
    f.render_widget(lessons_list, body_chunks[0]);

    // Render retro vertical scrollbar over total display rows
    let mut scrollbar_state = ScrollbarState::new(display_rows.len()).position(selected_display_idx);
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("▲"))
        .end_symbol(Some("▼"))
        .track_symbol(Some("│"))
        .thumb_symbol("█");
    f.render_stateful_widget(scrollbar, body_chunks[0], &mut scrollbar_state);

    // Sidebar: Pilot Log & Telemetry
    let total_mins = app.user_progress.total_practice_seconds / 60;
    let completed_count = app.user_progress.completed_lessons.values().filter(|v| v.passed).count();

    let rank_title = match app.user_progress.unlocked_tier {
        Tier::Tier1Foundation => "Cadete de Órbita (Tier 1)",
        Tier::Tier2FullAlphabet => "Navegante Alfa (Tier 2)",
        Tier::Tier3SpanishOrthography => "Piloto de Ortografía Estelar (Tier 3)",
        Tier::Tier4NumbersAndSymbols => "Especialista en Código (Tier 4)",
        Tier::Tier5SpeedAndCadence => "Capitán de Impulso Lumínico (Tier 5)",
        Tier::Tier6AdvancedFluency => "Comandante de Prosa Cuántica (Tier 6)",
        Tier::Tier7GrandMaster => "Gran Maestro del Hiperespacio (Tier 7 - 150 WPM)",
    };

    let sidebar_lines = vec![
        Line::from(vec![
            Span::styled("Rango Espacial: ", Style::default().fg(Theme::MUTED)),
            Span::styled(rank_title, Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Sectores Conquistados: ", Style::default().fg(Theme::MUTED)),
            Span::styled(format!("{}/{}", completed_count, lessons.len()), Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Horas de Vuelo: ", Style::default().fg(Theme::MUTED)),
            Span::styled(format!("{} minutos", total_mins), Style::default().fg(Theme::TEXT)),
        ]),
        Line::from(""),
        Line::from(Span::styled("✦ SENSORES DE IMPACTO (TECLAS CRÍTICAS) ✦", Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD))),
    ];

    let mut sidebar_all_lines = sidebar_lines;
    let mut weak_keys: Vec<_> = app.user_progress.key_stats.iter().collect();
    weak_keys.sort_by(|a, b| b.1.error_rate().partial_cmp(&a.1.error_rate()).unwrap_or(std::cmp::Ordering::Equal));

    let mut has_weak = false;
    for (ch, stat) in weak_keys.into_iter().take(4) {
        if stat.errors > 0 {
            has_weak = true;
            let display_char = if *ch == ' ' { "ESPACIO".to_string() } else { format!("'{}'", ch) };
            sidebar_all_lines.push(Line::from(vec![
                Span::styled(format!(" • Tecla {}: ", display_char), Style::default().fg(Theme::ERROR)),
                Span::styled(format!("{:.1}% fallo ({} err)", stat.error_rate(), stat.errors), Style::default().fg(Theme::MUTED)),
            ]));
        }
    }

    if !has_weak {
        sidebar_all_lines.push(Line::from(Span::styled(" • Sensores nominales (0 fallos registrados)", Style::default().fg(Theme::SUCCESS))));
    }

    let sidebar = Paragraph::new(sidebar_all_lines)
        .block(Theme::retro_block("✦ BITÁCORA DEL PILOTO & TELEMETRÍA ✦", Theme::NEBULA_PURPLE))
        .wrap(Wrap { trim: true });
    f.render_widget(sidebar, body_chunks[1]);

    // 3. Footer Keybinds
    let footer_spans = vec![
        Span::styled("[↑/↓ j/k] ", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
        Span::styled("Navegar  ", Style::default().fg(Theme::TEXT)),
        Span::styled("[PgUp/PgDn / g/G] ", Style::default().fg(Theme::NEBULA_PURPLE).add_modifier(Modifier::BOLD)),
        Span::styled("Scroll  ", Style::default().fg(Theme::TEXT)),
        Span::styled("[ENTER] ", Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)),
        Span::styled("Iniciar  ", Style::default().fg(Theme::TEXT)),
        Span::styled("[V] ", Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD)),
        Span::styled("Dictado  ", Style::default().fg(Theme::TEXT)),
        Span::styled("[D] ", Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled("Adaptativo  ", Style::default().fg(Theme::TEXT)),
        Span::styled("[E] ", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
        Span::styled("Telemetría  ", Style::default().fg(Theme::TEXT)),
        Span::styled("[Q] ", Style::default().fg(Theme::MUTED).add_modifier(Modifier::BOLD)),
        Span::styled("Salir", Style::default().fg(Theme::TEXT)),
    ];
    let footer = Paragraph::new(Line::from(footer_spans))
        .block(Theme::retro_block("MANDOS DE LA NAVE", Theme::MUTED))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}

fn render_dictation(f: &mut Frame, app: &App) {
    if let Some(engine) = &app.current_dictation {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),  // Header
                Constraint::Length(11), // Dictation Area (ribbon + slots + audio bar)
                Constraint::Min(10),   // Keyboard visualizer
                Constraint::Length(3),  // Footer
            ])
            .split(f.area());

        // 1. Header
        let header_lines = vec![
            Line::from(vec![
                Span::styled("✦ MODO DICTADO AUDITIVO · RECEPTOR SUBESPACIAL ✦ ", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
                Span::styled("— Entrenamiento de Reflejo Auditivo-Motor", Style::default().fg(Theme::TEXT)),
            ]),
            Line::from(vec![
                Span::styled(" Decodifica la transmisión de voz. ", Style::default().fg(Theme::SECONDARY)),
                Span::styled("Usa [TAB] si necesitas repetir la señal de audio.", Style::default().fg(Theme::MUTED)),
            ]),
        ];
        let header = Paragraph::new(header_lines)
            .block(Theme::retro_block("CANAL DE COMUNICACIÓN SUBESPACIAL", Theme::PRIMARY))
            .alignment(Alignment::Center);
        f.render_widget(header, chunks[0]);

        // 2. Dictation Area
        DictationArea::render(f, chunks[1], engine);

        // 3. Spanish ISO Keyboard Visualizer
        let target_char = engine.current_expected_char();
        let keyboard_widget = KeyboardVisualizer::render(target_char);
        f.render_widget(keyboard_widget, chunks[2]);

        // 4. Dictation Footer
        let footer_spans = vec![
            Span::styled("[ESC] ", Style::default().fg(Theme::MUTED).add_modifier(Modifier::BOLD)),
            Span::styled("Menú   ", Style::default().fg(Theme::TEXT)),
            Span::styled("[TAB] ", Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD)),
            Span::styled("Repetir Audio   ", Style::default().fg(Theme::TEXT)),
            Span::styled("[F2 / Shift+TAB] ", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled("Cambiar Frecuencia (Voz)   ", Style::default().fg(Theme::TEXT)),
            Span::styled("[+ / -] ", Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled("Velocidad Audio   ", Style::default().fg(Theme::TEXT)),
            Span::styled("Regla: ≥ 96% precisión", Style::default().fg(Theme::MUTED)),
        ];
        let footer = Paragraph::new(Line::from(footer_spans))
            .block(Theme::retro_block("MANDOS DE RECEPTOR", Theme::MUTED))
            .alignment(Alignment::Center);
        f.render_widget(footer, chunks[3]);
    }
}

fn render_practice(f: &mut Frame, app: &App) {
    if let Some(engine) = &app.current_engine {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Length(6),
                Constraint::Min(10),
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
            Span::styled("Abortar al Menú   ", Style::default().fg(Theme::TEXT)),
            Span::styled("[TAB] ", Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD)),
            Span::styled("Reiniciar Misión   ", Style::default().fg(Theme::TEXT)),
            Span::styled("✦ FILA GUÍA: Mantén dedos en ASDF - JKLÑ sin apartar la vista ✦", Style::default().fg(Theme::NEBULA_PURPLE)),
        ];
        let footer = Paragraph::new(Line::from(footer_spans))
            .block(Theme::retro_block("MANDOS DE VUELO", Theme::MUTED))
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
    lines.push(Line::from(Span::styled(
        "✦ CARTOGRAFÍA ESTELAR & MAPA DE RENDIMIENTO POR TECLA ✦",
        Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    let mut stats: Vec<_> = app.user_progress.key_stats.iter().collect();
    stats.sort_by_key(|b| std::cmp::Reverse(b.1.attempts));

    if stats.is_empty() {
        lines.push(Line::from(Span::styled("Aún no hay datos de vuelo registrados. ¡Completa misiones para generar tu cartografía estelar!", Style::default().fg(Theme::MUTED))));
    } else {
        lines.push(Line::from(vec![
            Span::styled(format!("{:<8} {:<12} {:<12} {:<15} {:<15}", "Tecla", "Intentos", "Impactos", "% Error", "Latencia Med."), Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(Span::styled("─".repeat(65), Style::default().fg(Theme::MUTED))));

        for (ch, stat) in stats.into_iter().take(14) {
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
        .block(Theme::retro_block("TELEMETRÍA HISTÓRICA DE VUELO", Theme::PRIMARY))
        .alignment(Alignment::Left);
    f.render_widget(stats_widget, chunks[0]);

    let footer_spans = vec![
        Span::styled("[D] ", Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD)),
        Span::styled("Iniciar Drill de Teclas Débiles   ", Style::default().fg(Theme::TEXT)),
        Span::styled("[ESC / ENTER / Q] ", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
        Span::styled("Volver al Comando Central", Style::default().fg(Theme::TEXT)),
    ];
    let footer = Paragraph::new(Line::from(footer_spans))
        .block(Theme::retro_block("ACCIONES", Theme::MUTED))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[1]);
}
