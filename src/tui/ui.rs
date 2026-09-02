use crate::core::model::{Lesson, PlanetStatus, Tier};
use crate::core::Curriculum;
use crate::tui::app::{App, CurrentView};
use crate::tui::ascii::AsciiArt;
use crate::tui::components::{
    DictationArea, DictationSummaryModal, KeyboardVisualizer, StatsBar, SummaryModal, TypingArea,
};
use crate::tui::planet_layout::{
    build_rows, display_index_of, map_mode, planet_layout, ship_gutter, ship_rect, viewport_start,
    MapMode, MenuRow,
};
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
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
        CurrentView::PlanetLessons => render_planet_lessons(f, app),
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

/// Retro ASCII header banner shared by [`render_main_menu`] and
/// [`render_planet_lessons`].
fn render_header(f: &mut Frame, area: Rect) {
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
    f.render_widget(header, area);
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

    render_header(f, chunks[0]);

    // 2. Body: Left = Sectors List, Right = Pilot Telemetry & Stats
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(chunks[1]);

    let lessons = app.available_lessons();

    // Galaxy tier map: one card/row per tier ("planet"), driven by aggregated
    // lesson counters. The ship gutter (Full mode only) renders the sprite
    // via `render_ship_sprite`.
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

    if is_full_mode {
        render_ship_sprite(f, body_chunks[0], &planet_rects, app);
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

/// Renders the animated ship sprite inside the galaxy map's ship gutter
/// ([`MapMode::Full`] only; the caller already checked the mode). Applies a
/// small vertical dip while [`crate::tui::animation::ShipAnimation`] is
/// descending/ascending, and colours the thruster line differently per
/// flicker frame.
fn render_ship_sprite(f: &mut Frame, map_area: Rect, planet_rects: &[Rect], app: &App) {
    let gutter = ship_gutter(map_area);
    let frame_idx = app.ship.frame_index();
    let frame = &AsciiArt::SHIP_FRAMES[frame_idx];
    let sprite_h = frame.len() as u16;
    let mut rect = ship_rect(gutter, planet_rects, app.ship.position(), sprite_h);
    if rect.width == 0 || rect.height == 0 {
        return;
    }

    let offset = app.ship.vertical_offset().round().min(1.0) as u16;
    let gutter_bottom = gutter.y.saturating_add(gutter.height);
    let max_y = gutter_bottom.saturating_sub(rect.height).max(gutter.y);
    rect.y = rect.y.saturating_add(offset).min(max_y);

    let lines: Vec<Line> = frame
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let is_thruster_line = i == frame.len() - 1;
            let color = if frame_idx == 1 && is_thruster_line { Theme::ERROR } else { Theme::ACCENT };
            Line::from(Span::styled(*line, Style::default().fg(color)))
        })
        .collect();
    f.render_widget(Paragraph::new(lines), rect);
}

/// Styled [`Line`] for one row of the planet lesson list. Headers render
/// dimmed/bold; lesson rows show a selection marker, a passed badge, the
/// title and target CPM.
fn menu_row_line(row: &MenuRow, app: &App, lessons: &[Lesson]) -> Line<'static> {
    match row {
        MenuRow::Header(title) => Line::from(Span::styled(
            title.clone(),
            Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD),
        )),
        MenuRow::Lesson(flat_idx) => {
            let lesson = &lessons[*flat_idx];
            let passed = app
                .user_progress
                .completed_lessons
                .get(&lesson.id)
                .map(|score| score.passed)
                .unwrap_or(false);
            let is_selected = *flat_idx == app.selected_lesson_index;

            let marker = if is_selected { "▶ " } else { "  " };
            let badge = if passed { "✔ " } else { "  " };
            let style = if is_selected {
                Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)
            } else if passed {
                Style::default().fg(Theme::SUCCESS)
            } else {
                Style::default().fg(Theme::TEXT)
            };

            let text = format!("{marker}{badge}{} · {:.0} CPM", lesson.title, lesson.target_cpm);
            Line::from(Span::styled(text, style))
        }
    }
}

fn render_planet_lessons(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(12), Constraint::Length(3)])
        .split(f.area());

    render_header(f, chunks[0]);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(chunks[1]);

    // Left panel: section-grouped lesson list for the selected tier.
    let tier = app.selected_tier();
    let lessons = app.available_lessons();
    let tier_lessons: Vec<(usize, &Lesson)> = lessons
        .iter()
        .enumerate()
        .filter(|(_, lesson)| lesson.tier == tier)
        .collect();
    let sections = Curriculum::all_sections();
    let rows = build_rows(&tier_lessons, &sections);

    let list_title = format!(" ◎ {} :: SECTORES ", tier.planet_name());
    let list_block = Theme::retro_block(&list_title, Theme::PRIMARY);
    let inner = list_block.inner(body_chunks[0]);
    f.render_widget(list_block, body_chunks[0]);

    if inner.height > 0 {
        let capacity = inner.height as usize;
        let selected_display = display_index_of(&rows, app.selected_lesson_index).unwrap_or(0);
        let start = viewport_start(selected_display, capacity);

        let lines: Vec<Line> = rows
            .iter()
            .skip(start)
            .take(capacity)
            .map(|row| menu_row_line(row, app, &lessons))
            .collect();
        f.render_widget(Paragraph::new(lines), inner);
    }

    // Right panel: compact tier telemetry.
    let tier_progress = Curriculum::all_tier_progress(&app.user_progress);
    let tp = &tier_progress[tier.index()];
    let sidebar_lines = vec![
        Line::from(Span::styled(tier.name(), Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Sectores conquistados: ", Style::default().fg(Theme::MUTED)),
            Span::styled(format!("{}/{}", tp.passed, tp.total), Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Velocidad mínima: ", Style::default().fg(Theme::MUTED)),
            Span::styled(format!("{:.0} CPM", tier.min_cpm()), Style::default().fg(Theme::TEXT)),
        ]),
    ];
    let sidebar = Paragraph::new(sidebar_lines)
        .block(Theme::retro_block("✦ TELEMETRÍA DEL SECTOR ✦", Theme::NEBULA_PURPLE))
        .wrap(Wrap { trim: true });
    f.render_widget(sidebar, body_chunks[1]);

    // Footer keybinds.
    let footer_spans = vec![
        Span::styled("[↑/↓ j/k] ", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
        Span::styled("Lección  ", Style::default().fg(Theme::TEXT)),
        Span::styled("[ENTER] ", Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)),
        Span::styled("Practicar  ", Style::default().fg(Theme::TEXT)),
        Span::styled("[ESC/←] ", Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD)),
        Span::styled("Volver al mapa  ", Style::default().fg(Theme::TEXT)),
        Span::styled("[Q] ", Style::default().fg(Theme::MUTED).add_modifier(Modifier::BOLD)),
        Span::styled("Volver", Style::default().fg(Theme::TEXT)),
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
    fn test_ship_sprite_renders_in_gutter_full_mode() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;

        // 120x40 is the smallest round size that actually reaches Full mode:
        // the map body is 62% of window width and MIN_FULL_HEIGHT (30) is
        // checked against `chunks[1].height` (window height minus the 6-row
        // header and 3-row footer), so 100x34 (used elsewhere in this test
        // module) stays Compact — verified via `map_mode` on the derived
        // body rect.
        let rendered_full = render_to_string(&app, 120, 40);
        assert!(
            rendered_full.contains("◄███►"),
            "expected ship sprite in Full mode gutter:\n{rendered_full}"
        );

        let rendered_compact = render_to_string(&app, 40, 12);
        assert!(
            !rendered_compact.contains("◄███►"),
            "compact mode must not render the ship sprite:\n{rendered_compact}"
        );
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

    #[test]
    fn test_planet_lessons_view_lists_only_selected_tier() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        app.enter_planet_lessons();

        let tier1_title = app
            .available_lessons()
            .into_iter()
            .find(|l| l.tier == Tier::Tier1Foundation)
            .expect("fixture needs a Tier1 lesson")
            .title;
        let tier2_title = app
            .available_lessons()
            .into_iter()
            .find(|l| l.tier == Tier::Tier2FullAlphabet)
            .expect("fixture needs a Tier2 lesson")
            .title;

        let rendered = render_to_string(&app, 100, 34);

        assert!(rendered.contains(&tier1_title), "expected Tier1 lesson '{tier1_title}' in:\n{rendered}");
        assert!(rendered.contains(Tier::Tier1Foundation.planet_name()), "expected Tier1 planet name in:\n{rendered}");
        assert!(!rendered.contains(&tier2_title), "must not list Tier2 lesson '{tier2_title}' while on Tier1:\n{rendered}");
    }

    #[test]
    fn test_planet_lessons_view_1x1_no_panic() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        app.enter_planet_lessons();

        let rendered = render_to_string(&app, 1, 1);
        assert_eq!(rendered.chars().count(), 1);
    }

    #[test]
    fn test_planet_lessons_view_shows_section_header() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        app.enter_planet_lessons();

        let section_title = Curriculum::all_sections()
            .into_iter()
            .find(|s| s.tier == Tier::Tier1Foundation)
            .expect("fixture needs a Tier1 section")
            .title;

        let rendered = render_to_string(&app, 100, 34);

        assert!(rendered.contains(&section_title), "expected section header '{section_title}' in:\n{rendered}");
    }
}
