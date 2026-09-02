use crate::core::model::{PlanetStatus, Tier};
use crate::core::Curriculum;
use crate::tui::app::{App, CurrentView};
use crate::tui::ascii::AsciiArt;
use crate::tui::components::{
    DictationArea, DictationSummaryModal, KeyboardVisualizer, StatsBar, SummaryModal, TypingArea,
};
use crate::tui::planet_layout::{map_mode, planet_layout, MapMode};
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

/// Maps a [`PlanetStatus`] to its galaxy-map glyph, colour, and Spanish
/// status badge. Colour is never the sole indicator: the glyph and badge
/// text are always present alongside it.
fn planet_style(status: PlanetStatus) -> (&'static str, Color, &'static str) {
    match status {
        PlanetStatus::Conquered => ("◉", Theme::SUCCESS, "CONQUISTADO"),
        PlanetStatus::Current => ("◎", Theme::PRIMARY, "DESTINO ACTUAL"),
        PlanetStatus::InProgress => ("◍", Theme::ACCENT, "EN CURSO"),
        PlanetStatus::Unexplored => ("○", Theme::MUTED, "SIN EXPLORAR"),
    }
}

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
        // replaced by render_planet_lessons in PR5
        CurrentView::PlanetLessons => render_main_menu(f, app),
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

    // Galaxy tier map: one card/row per tier ("planet"), driven by aggregated
    // lesson counters. The ship gutter (Full mode only) stays empty until the
    // sprite lands in PR7.
    let tier_progress = Curriculum::all_tier_progress(&app.user_progress);
    let planet_rects = planet_layout(body_chunks[0]);
    let is_full_mode = map_mode(body_chunks[0]) == MapMode::Full;

    for (i, (tp, rect)) in tier_progress.iter().zip(planet_rects.iter()).enumerate() {
        if rect.width == 0 || rect.height == 0 {
            continue;
        }

        let (glyph, color, badge) = planet_style(tp.status);
        let is_selected = i == app.selected_planet_index;
        let text_style = if tp.status == PlanetStatus::Unexplored {
            Style::default().fg(Theme::MUTED)
        } else {
            Style::default().fg(Theme::TEXT)
        };

        if is_full_mode {
            let prefix = if is_selected { "▶ " } else { "  " };
            let title = format!(" {}{} {} ── {} ", prefix, glyph, tp.tier.planet_name(), badge);
            let mut border_style = Style::default().fg(color);
            if is_selected {
                border_style = border_style.add_modifier(Modifier::BOLD);
            }
            let block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style);
            let content = format!(
                "{}/{} sectores · meta {:.0} CPM · {}",
                tp.passed,
                tp.total,
                tp.tier.min_cpm(),
                tp.tier.name()
            );
            let paragraph = Paragraph::new(Line::from(Span::styled(content, text_style))).block(block);
            f.render_widget(paragraph, *rect);
        } else {
            let prefix = if is_selected { "▶ " } else { "  " };
            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} ", glyph), Style::default().fg(color)),
                Span::styled(format!("{:<10}  ", tp.tier.planet_name()), text_style),
                Span::styled(format!("{}/{}  ", tp.passed, tp.total), Style::default().fg(Theme::ACCENT)),
                Span::styled(badge, Style::default().fg(color)),
            ]);
            f.render_widget(Paragraph::new(line), *rect);
        }
    }

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
        Span::styled("Planeta  ", Style::default().fg(Theme::TEXT)),
        Span::styled("[ENTER] ", Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)),
        Span::styled("Explorar  ", Style::default().fg(Theme::TEXT)),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::{PlanetStatus, UserProgress};
    use ratatui::{backend::TestBackend, Terminal};

    /// Renders `app` into a `width x height` `TestBackend` buffer and returns
    /// every cell symbol concatenated into a single string, so plain
    /// `contains` checks can assert on rendered content.
    fn render_to_string(app: &App, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, app)).unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }

    #[test]
    fn test_100x34_shows_cimientos_and_sin_explorar_badges() {
        let mut app = App::new();
        // Deterministic progress: Tier1 unlocked, nothing completed yet, so
        // the galaxy map always shows CIMIENTOS as the current destination
        // and every other planet as unexplored regardless of the player's
        // real saved progress file.
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;

        let rendered = render_to_string(&app, 100, 34);

        assert!(rendered.contains("CIMIENTOS"), "expected CIMIENTOS planet name in:\n{rendered}");
        assert!(rendered.contains("SIN EXPLORAR"), "expected SIN EXPLORAR badge in:\n{rendered}");
    }

    #[test]
    fn test_40x12_compact_shows_7_names_no_sprite_no_panic() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;

        let rendered = render_to_string(&app, 40, 12);

        for tier in Tier::ALL {
            assert!(
                rendered.contains(tier.planet_name()),
                "expected planet name {} in:\n{rendered}",
                tier.planet_name()
            );
        }
        assert!(!rendered.contains('▲'), "compact mode must not render the ship sprite:\n{rendered}");
        assert!(!rendered.contains('█'), "compact mode must not render the ship sprite:\n{rendered}");
    }

    #[test]
    fn test_1x1_area_does_not_panic() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;

        // Must not panic; the assertion below is a secondary sanity check.
        let rendered = render_to_string(&app, 1, 1);
        assert_eq!(rendered.chars().count(), 1);
    }

    #[test]
    fn test_status_badges_match_precedence_for_each_glyph() {
        assert_eq!(planet_style(PlanetStatus::Conquered), ("◉", Theme::SUCCESS, "CONQUISTADO"));
        assert_eq!(planet_style(PlanetStatus::Current), ("◎", Theme::PRIMARY, "DESTINO ACTUAL"));
        assert_eq!(planet_style(PlanetStatus::InProgress), ("◍", Theme::ACCENT, "EN CURSO"));
        assert_eq!(planet_style(PlanetStatus::Unexplored), ("○", Theme::MUTED, "SIN EXPLORAR"));
    }
}
