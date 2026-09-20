use crate::core::Curriculum;
use crate::core::model::{Lesson, PlanetStatus, SessionKind, SessionRecord, Tier};
use crate::tui::animation::ShipPhase;
use crate::tui::app::{App, CurrentView};
use crate::tui::ascii::AsciiArt;
use crate::tui::components::{
    DictationArea, DictationSummaryModal, KeyboardVisualizer, StatsBar, SummaryModal, TypingArea,
    render_map_card, render_observatory,
};
use crate::tui::planet_layout::{
    MapMode, MenuRow, band_viewport_start, display_index_of, info_column, map_mode, planet_layout,
    ship_gutter, ship_x, ship_y, sprite_lane,
};
use crate::tui::theme::Theme;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

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
            if let (Some(engine), Some(metrics)) = (&app.current_engine, &app.last_session_metrics)
            {
                SummaryModal::render(
                    f,
                    f.area(),
                    &engine.lesson,
                    metrics,
                    app.last_session_passed,
                );
            }
        }
        CurrentView::Stats => render_stats(f, app),
        CurrentView::History => render_history(f, app),
        CurrentView::PlanetLessons => render_planet_lessons(f, app),
        CurrentView::Observatory => render_observatory_view(f, app),
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

/// Full-screen docked-planet view ([`CurrentView::Observatory`]): the
/// large ray-cast sphere with rings, moon, and telemetry HUD for the
/// currently selected planet. Opened by confirming a docked ship
/// (three-step Enter: descend → confirm → observatory); its Enter opens
/// the lessons, its Esc ascends back to the map.
fn render_observatory_view(f: &mut Frame, app: &App) {
    let tier_idx = app.selected_planet_index;
    let tier_progress = Curriculum::all_tier_progress(&app.user_progress);
    if let Some(tp) = tier_progress.get(tier_idx) {
        render_observatory(
            f,
            f.area(),
            tier_idx,
            tp.tier.planet_name(),
            tp.status,
            app.ship.total_elapsed(),
            app.reduced_motion,
        );
    }
}

/// Retro ASCII header banner shared by [`render_main_menu`] and
/// [`render_planet_lessons`].
fn render_header(f: &mut Frame, area: Rect) {
    let header_lines = vec![
        Line::from(Span::styled(
            AsciiArt::LOGO_LINES[0],
            Style::default()
                .fg(Theme::PRIMARY)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            AsciiArt::LOGO_LINES[1],
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            AsciiArt::SUBTITLE,
            Style::default()
                .fg(Theme::NEBULA_PURPLE)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                "✦ REGLA DE ORO ESTELAR: ",
                Style::default()
                    .fg(Theme::ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Precisión obligatoria ≥ 96% para desbloquear sectores de navegación estelar",
                Style::default().fg(Theme::TEXT),
            ),
        ]),
    ];
    let header = Paragraph::new(header_lines)
        .block(Theme::retro_block(
            "COMANDO CENTRAL :: MECANOPRO",
            Theme::PRIMARY,
        ))
        .alignment(Alignment::Center);
    f.render_widget(header, area);
}

/// Rect of the galaxy map body (left panel of the [`CurrentView::MainMenu`]
/// body row) for a full-window `frame`. Reproduces the exact `Layout` math
/// [`render_main_menu`] uses to place the planet cards, so mouse
/// hit-testing (`EventHandler::handle_mouse`) can never drift from render.
pub fn map_body_area(frame: Rect) -> Rect {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Min(12),
            Constraint::Length(3),
        ])
        .split(frame);
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(chunks[1]);
    body_chunks[0]
}

/// Rect of the planet lesson list block for a full-window `frame`.
/// [`render_planet_lessons`] uses the identical outer chunking as
/// [`render_main_menu`], so this currently reuses [`map_body_area`]; kept as
/// its own named function so hit-testing reads intent, not coincidence.
pub fn lesson_list_area(frame: Rect) -> Rect {
    map_body_area(frame)
}

/// Truncates `s` to fit `max_width` display columns, appending an ellipsis.
fn ellipsize(s: &str, max_width: u16) -> String {
    if max_width == 0 {
        return String::new();
    }
    if UnicodeWidthStr::width(s) as u16 <= max_width {
        return s.to_string();
    }
    let mut out = String::new();
    let mut w = 0usize;
    for ch in s.chars() {
        let cw = ch.width().unwrap_or(0);
        if w + cw > max_width.saturating_sub(1) as usize {
            break;
        }
        out.push(ch);
        w += cw;
    }
    format!("{out}…")
}

/// Pure content of one Full-mode planet card's info block: `[name, status
/// line, progress·meta line]` stacked below the sprite lane (D8). Degrades
/// monotonically with `width`: (1) the trailing CPM meta detail truncates
/// first (the progress digits lead the line, so they never drop), (2) the
/// planet name is ellipsized. The status glyph and the badge are never
/// dropped.
fn info_column_content(
    name: &str,
    glyph: &str,
    badge: &str,
    progress: &str,
    meta: Option<&str>,
    width: u16,
) -> [String; 3] {
    [
        ellipsize(name, width),
        format!("{glyph} {badge}"),
        match meta {
            Some(m) => ellipsize(m, width),
            None => progress.to_string(),
        },
    ]
}

/// Pre-formatted text pieces of one Full-mode planet row's info column.
struct InfoRow<'a> {
    /// Display name; the selection marker `▶` is already prefixed.
    name: &'a str,
    glyph: &'a str,
    color: Color,
    badge: &'a str,
    progress: &'a str,
    meta: &'a str,
}

/// Renders the info column of a Full-mode planet row, delegating the content
/// degrade to [`info_column_content`]. Selection bolds the whole column.
fn render_info_column(
    f: &mut Frame,
    area: Rect,
    row: &InfoRow,
    is_selected: bool,
    text_style: Style,
) {
    let base_color = text_style.fg.unwrap_or(Theme::TEXT);
    let content = info_column_content(
        row.name,
        row.glyph,
        row.badge,
        row.progress,
        Some(row.meta),
        area.width,
    );
    let lines: Vec<Line> = content
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let line_color = if i == 1 {
                row.color
            } else if i == 2 {
                Theme::ACCENT
            } else {
                base_color
            };
            let style = Style::default().fg(line_color);
            let style = if is_selected {
                style.add_modifier(Modifier::BOLD)
            } else {
                style
            };
            Line::from(Span::styled(text.clone(), style))
        })
        .collect();
    f.render_widget(Paragraph::new(lines), area);
}

fn render_main_menu(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Retro ASCII Header
            Constraint::Min(12),   // Body (Sectors & Telemetry)
            Constraint::Length(3), // Retro Footer
        ])
        .split(f.area());

    render_header(f, chunks[0]);

    // 2. Body: Left = Sectors List, Right = Pilot Telemetry & Stats
    let map_area = map_body_area(f.area());
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(chunks[1]);

    let lessons = app.available_lessons();

    // Galaxy tier map: horizontal Tier1→7 band ("planet" cards), driven by
    // aggregated lesson counters. The top ship-gutter row (Full mode only)
    // renders the sprite via `render_ship_sprite`; the camera window pans
    // so the selected planet stays visible.
    let tier_progress = Curriculum::all_tier_progress(&app.user_progress);
    let planet_rects = planet_layout(map_area, app.selected_planet_index);
    // Ray-cast spheres: each lane's disc rotates with the ship's monotonic
    // mission clock at the tier's own config period (D10: reduced motion
    // pins the phase at t=0 inside the widget).
    let is_full_mode = map_mode(map_area) == MapMode::Full;

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
            // Borderless card (D8 stacked anatomy): sphere lane on top,
            // info lines below. Deliberately no `Block`/`Borders` — the
            // card rect must stay free of ╭╮╰╯ chrome.
            let lane = sprite_lane(*rect);
            let info = info_column(*rect);
            render_map_card(
                f.buffer_mut(),
                lane,
                i,
                tp.status,
                app.ship.total_elapsed(),
                app.reduced_motion,
            );

            let display_name = if is_selected {
                format!("▶ {}", tp.tier.planet_name())
            } else {
                tp.tier.planet_name().to_string()
            };
            let progress = format!("{}/{}", tp.passed, tp.total);
            let meta = format!("{progress} sectores · meta {:.0} CPM", tp.tier.min_cpm());
            let row = InfoRow {
                name: &display_name,
                glyph,
                color,
                badge,
                progress: &progress,
                meta: &meta,
            };
            render_info_column(f, info, &row, is_selected, text_style);
        } else {
            let prefix = if is_selected { "▶ " } else { "  " };
            let line = Line::from(vec![
                Span::styled(
                    prefix,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("{} ", glyph), Style::default().fg(color)),
                Span::styled(format!("{:<10}  ", tp.tier.planet_name()), text_style),
                Span::styled(
                    format!("{}/{}  ", tp.passed, tp.total),
                    Style::default().fg(Theme::ACCENT),
                ),
                Span::styled(badge, Style::default().fg(color)),
            ]);
            f.render_widget(Paragraph::new(line), *rect);
        }
    }

    if is_full_mode {
        render_ship_sprite(f, map_area, &planet_rects, app);
    }

    // Sidebar: Pilot Log & Telemetry
    let total_mins = app.user_progress.total_practice_seconds / 60;
    let completed_count = app
        .user_progress
        .completed_lessons
        .values()
        .filter(|v| v.passed)
        .count();

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
            Span::styled(
                rank_title,
                Style::default()
                    .fg(Theme::ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Sectores Conquistados: ", Style::default().fg(Theme::MUTED)),
            Span::styled(
                format!("{}/{}", completed_count, lessons.len()),
                Style::default()
                    .fg(Theme::SUCCESS)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Horas de Vuelo: ", Style::default().fg(Theme::MUTED)),
            Span::styled(
                format!("{} minutos", total_mins),
                Style::default().fg(Theme::TEXT),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "✦ SENSORES DE IMPACTO (TECLAS CRÍTICAS) ✦",
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        )),
    ];

    let mut sidebar_all_lines = sidebar_lines;
    let mut weak_keys: Vec<_> = app.user_progress.key_stats.iter().collect();
    weak_keys.sort_by(|a, b| {
        b.1.error_rate()
            .partial_cmp(&a.1.error_rate())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut has_weak = false;
    for (ch, stat) in weak_keys.into_iter().take(4) {
        if stat.errors > 0 {
            has_weak = true;
            let display_char = if *ch == ' ' {
                "ESPACIO".to_string()
            } else {
                format!("'{}'", ch)
            };
            sidebar_all_lines.push(Line::from(vec![
                Span::styled(
                    format!(" • Tecla {}: ", display_char),
                    Style::default().fg(Theme::ERROR),
                ),
                Span::styled(
                    format!("{:.1}% fallo ({} err)", stat.error_rate(), stat.errors),
                    Style::default().fg(Theme::MUTED),
                ),
            ]));
        }
    }

    if !has_weak {
        sidebar_all_lines.push(Line::from(Span::styled(
            " • Sensores nominales (0 fallos registrados)",
            Style::default().fg(Theme::SUCCESS),
        )));
    }

    let sidebar = Paragraph::new(sidebar_all_lines)
        .block(Theme::retro_block(
            "✦ BITÁCORA DEL PILOTO & TELEMETRÍA ✦",
            Theme::NEBULA_PURPLE,
        ))
        .wrap(Wrap { trim: true });
    f.render_widget(sidebar, body_chunks[1]);

    // 3. Footer Keybinds
    let footer_spans = vec![
        Span::styled(
            "[←/→ h/l] ",
            Style::default()
                .fg(Theme::PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Planeta  ", Style::default().fg(Theme::TEXT)),
        Span::styled(
            "[ENTER] ",
            Style::default()
                .fg(Theme::SUCCESS)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Aterrizar/Abrir  ", Style::default().fg(Theme::TEXT)),
        Span::styled(
            "[V] ",
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Dictado  ", Style::default().fg(Theme::TEXT)),
        Span::styled(
            "[D] ",
            Style::default()
                .fg(Theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Adaptativo  ", Style::default().fg(Theme::TEXT)),
        Span::styled(
            "[E] ",
            Style::default()
                .fg(Theme::PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Telemetría  ", Style::default().fg(Theme::TEXT)),
        Span::styled(
            "[Q] ",
            Style::default()
                .fg(Theme::MUTED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Salir", Style::default().fg(Theme::TEXT)),
    ];
    let footer = Paragraph::new(Line::from(footer_spans))
        .block(Theme::retro_block("MANDOS DE LA NAVE", Theme::MUTED))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}

/// Renders the animated ship sprite on the galaxy map ([`MapMode::Full`]
/// only; the caller already checked the mode). The ship flies along the
/// top gutter row: [`ship_x`] lerps its center column between the
/// straddled cards' centers by the fractional planet position, while
/// [`ship_y`] holds the sprite-lane middle row and dips down onto the
/// docked card's top row by `dock_depth` while [`ShipPhase::Descending`]
/// (reversing while [`ShipPhase::Ascending`]). A WARP_TRAIL underlay
/// streaks behind the ship (left of it) during [`ShipPhase::Traveling`],
/// flicker-synced with the thruster frames.
fn render_ship_sprite(f: &mut Frame, map_area: Rect, planet_rects: &[Rect], app: &App) {
    let gutter = ship_gutter(map_area);
    // D10: reduced motion pins the thruster flicker at frame 0; full mode
    // cycles with the 120ms flicker clock.
    let frame_idx = if app.reduced_motion {
        0
    } else {
        app.ship.frame_index()
    };
    let frame = &AsciiArt::SHIP_FRAMES[frame_idx];
    let sprite_h = frame.len() as u16;
    let frame_w = AsciiArt::SHIP_FRAME_WIDTH as u16;

    if planet_rects.iter().all(|r| r.width == 0 || r.height == 0) {
        return; // degenerate layout: no visible planet to fly over
    }

    // D9 docked render: a docked ship renders as a fully-settled descend —
    // ship_y maps Descending + dock_depth 1.0 onto the card's top row.
    // Passing the real (Idle) phase would park it back in the gutter,
    // because `ship_y` short-circuits Idle to the lane core row and
    // `dock_depth()` itself returns 0.0 when idle.
    let (phase, dock_depth) = if app.is_docked() {
        (ShipPhase::Descending, 1.0)
    } else {
        (app.ship.phase(), app.ship.dock_depth())
    };

    let center_x = ship_x(planet_rects, app.ship.position());
    let center_y = ship_y(
        gutter,
        planet_rects,
        app.selected_planet_index,
        phase,
        dock_depth,
    );

    // Center → top-left conversion, clamped inside the map body so the
    // sprite and its trail can never spill into the header or sidebar.
    let max_x = map_area
        .x
        .saturating_add(map_area.width)
        .saturating_sub(frame_w)
        .max(map_area.x);
    let left = if center_x.is_finite() {
        (center_x - frame_w as f32 / 2.0).round() as u16
    } else {
        map_area.x
    };
    let rect_x = left.clamp(map_area.x, max_x);

    let max_y = map_area
        .y
        .saturating_add(map_area.height)
        .saturating_sub(sprite_h)
        .max(map_area.y);
    let top = if center_y.is_finite() {
        (center_y - sprite_h as f32 / 2.0).round() as u16
    } else {
        map_area.y
    };
    let rect_y = top.clamp(map_area.y, max_y);

    let rect = Rect::new(rect_x, rect_y, frame_w, sprite_h);

    // WARP_TRAIL underlay: horizontal streaks back along the flight lane
    // behind the ship, flicker-synced with the thruster frames while the
    // ship is mid-flight.
    if app.ship.phase() == ShipPhase::Traveling {
        let trail = &AsciiArt::WARP_TRAIL[frame_idx];
        let trail_x = (i32::from(rect.x) - i32::from(frame_w)).max(i32::from(map_area.x)) as u16;
        let trail_rect = Rect::new(trail_x, rect.y, frame_w, rect.height);
        let trail_lines: Vec<Line> = trail
            .iter()
            .map(|row| Line::from(Span::styled(*row, Style::default().fg(Theme::ACCENT))))
            .collect();
        f.render_widget(Paragraph::new(trail_lines), trail_rect);
    }

    let lines: Vec<Line> = frame
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let is_thruster_line = i == frame.len() - 1;
            let color = if frame_idx == 1 && is_thruster_line {
                Theme::ERROR
            } else {
                Theme::ACCENT
            };
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
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
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
                Style::default()
                    .fg(Theme::PRIMARY)
                    .add_modifier(Modifier::BOLD)
            } else if passed {
                Style::default().fg(Theme::SUCCESS)
            } else {
                Style::default().fg(Theme::TEXT)
            };

            let text = format!(
                "{marker}{badge}{} · {:.0} CPM",
                lesson.title, lesson.target_cpm
            );
            Line::from(Span::styled(text, style))
        }
    }
}

fn render_planet_lessons(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Min(12),
            Constraint::Length(3),
        ])
        .split(f.area());

    render_header(f, chunks[0]);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(chunks[1]);

    // Left panel: section-grouped lesson list for the selected tier. `rows`
    // is shared with `EventHandler::handle_mouse` via `App::current_tier_rows`
    // so click hit-testing can never see a different row order than render.
    let tier = app.selected_tier();
    let lessons = app.available_lessons();
    let rows = app.current_tier_rows();

    let list_area = lesson_list_area(f.area());
    let list_title = format!(" ◎ {} :: SECTORES ", tier.planet_name());
    let list_block = Theme::retro_block(&list_title, Theme::PRIMARY);
    let inner = list_block.inner(list_area);
    f.render_widget(list_block, list_area);

    if inner.height > 0 {
        let capacity = inner.height as usize;
        let selected_display = display_index_of(&rows, app.selected_lesson_index).unwrap_or(0);
        let start = band_viewport_start(selected_display, capacity);

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
        Line::from(Span::styled(
            tier.name(),
            Style::default()
                .fg(Theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Sectores conquistados: ", Style::default().fg(Theme::MUTED)),
            Span::styled(
                format!("{}/{}", tp.passed, tp.total),
                Style::default()
                    .fg(Theme::SUCCESS)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Velocidad mínima: ", Style::default().fg(Theme::MUTED)),
            Span::styled(
                format!("{:.0} CPM", tier.min_cpm()),
                Style::default().fg(Theme::TEXT),
            ),
        ]),
    ];
    let sidebar = Paragraph::new(sidebar_lines)
        .block(Theme::retro_block(
            "✦ TELEMETRÍA DEL SECTOR ✦",
            Theme::NEBULA_PURPLE,
        ))
        .wrap(Wrap { trim: true });
    f.render_widget(sidebar, body_chunks[1]);

    // Footer keybinds.
    let footer_spans = vec![
        Span::styled(
            "[↑/↓ j/k] ",
            Style::default()
                .fg(Theme::PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Lección  ", Style::default().fg(Theme::TEXT)),
        Span::styled(
            "[ENTER] ",
            Style::default()
                .fg(Theme::SUCCESS)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Practicar  ", Style::default().fg(Theme::TEXT)),
        Span::styled(
            "[ESC/←] ",
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Volver al mapa  ", Style::default().fg(Theme::TEXT)),
        Span::styled(
            "[Q] ",
            Style::default()
                .fg(Theme::MUTED)
                .add_modifier(Modifier::BOLD),
        ),
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
                Constraint::Min(10),    // Keyboard visualizer
                Constraint::Length(3),  // Footer
            ])
            .split(f.area());

        // 1. Header
        let header_lines = vec![
            Line::from(vec![
                Span::styled(
                    "✦ MODO DICTADO AUDITIVO · RECEPTOR SUBESPACIAL ✦ ",
                    Style::default()
                        .fg(Theme::PRIMARY)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "— Entrenamiento de Reflejo Auditivo-Motor",
                    Style::default().fg(Theme::TEXT),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    " Decodifica la transmisión de voz. ",
                    Style::default().fg(Theme::SECONDARY),
                ),
                Span::styled(
                    "Usa [TAB] si necesitas repetir la señal de audio.",
                    Style::default().fg(Theme::MUTED),
                ),
            ]),
        ];
        let header = Paragraph::new(header_lines)
            .block(Theme::retro_block(
                "CANAL DE COMUNICACIÓN SUBESPACIAL",
                Theme::PRIMARY,
            ))
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
            Span::styled(
                "[ESC] ",
                Style::default()
                    .fg(Theme::MUTED)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Menú   ", Style::default().fg(Theme::TEXT)),
            Span::styled(
                "[TAB] ",
                Style::default()
                    .fg(Theme::SECONDARY)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Repetir Audio   ", Style::default().fg(Theme::TEXT)),
            Span::styled(
                "[F2 / Shift+TAB] ",
                Style::default()
                    .fg(Theme::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Cambiar Frecuencia (Voz)   ",
                Style::default().fg(Theme::TEXT),
            ),
            Span::styled(
                "[+ / -] ",
                Style::default()
                    .fg(Theme::ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
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
            Span::styled(
                "[ESC] ",
                Style::default()
                    .fg(Theme::MUTED)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Abortar al Menú   ", Style::default().fg(Theme::TEXT)),
            Span::styled(
                "[TAB] ",
                Style::default()
                    .fg(Theme::SECONDARY)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Reiniciar Misión   ", Style::default().fg(Theme::TEXT)),
            Span::styled(
                "✦ FILA GUÍA: Mantén dedos en ASDF - JKLÑ sin apartar la vista ✦",
                Style::default().fg(Theme::NEBULA_PURPLE),
            ),
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
        Style::default()
            .fg(Theme::PRIMARY)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    let mut stats: Vec<_> = app.user_progress.key_stats.iter().collect();
    stats.sort_by_key(|b| std::cmp::Reverse(b.1.attempts));

    if stats.is_empty() {
        lines.push(Line::from(Span::styled("Aún no hay datos de vuelo registrados. ¡Completa misiones para generar tu cartografía estelar!", Style::default().fg(Theme::MUTED))));
    } else {
        lines.push(Line::from(vec![Span::styled(
            format!(
                "{:<8} {:<12} {:<12} {:<15} {:<15}",
                "Tecla", "Intentos", "Impactos", "% Error", "Latencia Med."
            ),
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(Span::styled(
            "─".repeat(65),
            Style::default().fg(Theme::MUTED),
        )));

        for (ch, stat) in stats.into_iter().take(14) {
            let error_color = if stat.error_rate() > 4.0 {
                Theme::ERROR
            } else {
                Theme::SUCCESS
            };
            let display_char = if *ch == ' ' {
                "ESPACIO".to_string()
            } else {
                format!("'{}'", ch)
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:<8} ", display_char),
                    Style::default().fg(Theme::TEXT),
                ),
                Span::styled(
                    format!("{:<12} ", stat.attempts),
                    Style::default().fg(Theme::MUTED),
                ),
                Span::styled(
                    format!("{:<12} ", stat.errors),
                    Style::default().fg(Theme::MUTED),
                ),
                Span::styled(
                    format!("{:<14.1}% ", stat.error_rate()),
                    Style::default()
                        .fg(error_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:.0} ms", stat.avg_latency_ms()),
                    Style::default().fg(Theme::TEXT),
                ),
            ]));
        }
    }

    let stats_widget = Paragraph::new(lines)
        .block(Theme::retro_block(
            "TELEMETRÍA HISTÓRICA DE VUELO",
            Theme::PRIMARY,
        ))
        .alignment(Alignment::Left);
    f.render_widget(stats_widget, chunks[0]);

    let footer_spans = vec![
        Span::styled(
            "[D] ",
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Iniciar Drill de Teclas Débiles   ",
            Style::default().fg(Theme::TEXT),
        ),
        Span::styled(
            "[ESC / ENTER / Q] ",
            Style::default()
                .fg(Theme::PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Volver al Comando Central",
            Style::default().fg(Theme::TEXT),
        ),
    ];
    let footer = Paragraph::new(Line::from(footer_spans))
        .block(Theme::retro_block("ACCIONES", Theme::MUTED))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[1]);
}

/// Renders the session-history view (`CurrentView::History`), reached from
/// `MainMenu` via `b`/`B`.
///
/// States covered: **ideal** (populated list) and **empty** (fresh install,
/// no sessions yet). Loading/error/partial states are deliberately not
/// modeled here — unlike an async fetch, `app.user_progress` is already
/// fully loaded into memory by `App::with_repository` before any frame is
/// drawn, so there is no in-flight or failable read at render time.
///
/// Reads exclusively through [`crate::core::model::UserProgress::sessions_recent_first`]
/// — never `app.user_progress.sessions` directly — so entries always appear
/// most-recent-first regardless of storage/retention order (design D7).
///
/// Deliberately out of scope (spec's history-view negative scenarios,
/// design's content-scope requirement): no confusion/substitution matrix, no
/// per-finger latency breakdown, no net-WPM figure, no export/import
/// control. That data is stored (`SessionSummary::net_wpm`, `SessionBucket`,
/// etc.) but intentionally not surfaced by this view.
fn render_history(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(f.area());

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "✦ BITÁCORA DE VUELO — HISTORIAL DE SESIONES ✦",
        Style::default()
            .fg(Theme::PRIMARY)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    let sessions = app.user_progress.sessions_recent_first();

    if sessions.is_empty() {
        lines.push(Line::from(Span::styled(
            "Aún no hay sesiones en tu bitácora. Completa una lección, un drill o un dictado para comenzar tu historial de vuelo.",
            Style::default().fg(Theme::MUTED),
        )));
    } else {
        for record in sessions {
            lines.push(session_history_line(record));
        }
    }

    let history_widget = Paragraph::new(lines)
        .block(Theme::retro_block("HISTORIAL DE SESIONES", Theme::PRIMARY))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(history_widget, chunks[0]);

    let footer_spans = vec![
        Span::styled(
            "[ESC / ENTER / Q] ",
            Style::default()
                .fg(Theme::PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Volver al Comando Central",
            Style::default().fg(Theme::TEXT),
        ),
    ];
    let footer = Paragraph::new(Line::from(footer_spans))
        .block(Theme::retro_block("ACCIONES", Theme::MUTED))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[1]);
}

/// One rendered line for a single [`SessionRecord`], with kind-specific
/// detail. Never reads `summary.net_wpm` — see [`render_history`]'s
/// content-scope note.
fn session_history_line(record: &SessionRecord) -> Line<'static> {
    let (kind_label, detail) = match &record.kind {
        SessionKind::Lesson {
            lesson_id, passed, ..
        } => {
            let result = if *passed {
                "✓ Aprobado"
            } else {
                "✗ No aprobado"
            };
            ("LECCIÓN".to_string(), format!("{lesson_id} — {result}"))
        }
        SessionKind::Drill => (
            "DRILL".to_string(),
            "Práctica de teclas débiles".to_string(),
        ),
        SessionKind::Dictation {
            completed_words,
            total_words,
            ..
        } => (
            "DICTADO".to_string(),
            format!("{completed_words}/{total_words} palabras"),
        ),
    };

    let summary = &record.summary;
    Line::from(vec![
        Span::styled(
            format!("• {kind_label:<8} "),
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("{detail} — "), Style::default().fg(Theme::TEXT)),
        Span::styled(
            format!(
                "{:.0} CPM · {:.1}% precisión · {}s",
                summary.cpm, summary.accuracy, record.duration_secs
            ),
            Style::default().fg(Theme::MUTED),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::{
        PlanetStatus, SessionKind, SessionRecord, SessionSummary, UserProgress,
    };
    use crate::tui::planet_layout::{PLANET_CARD_WIDTH, SHIP_GUTTER_HEIGHT, planet_at};
    use crate::tui::planets::RAMP;
    use ratatui::{Terminal, backend::TestBackend};
    use std::time::Duration;

    /// True when `symbols` contains at least one sphere ramp glyph
    /// (`·░▒▓█`) — the map-card lane contract painted by
    /// [`crate::tui::components::render_map_card`].
    fn contains_ramp_glyph(symbols: &str) -> bool {
        RAMP.iter().any(|ramp| symbols.contains(*ramp))
    }

    /// Counts cells painted with the bright half of the shading ramp
    /// (steps 2–4, the day-lit cells): the day region a Conquered sun
    /// lights up on a card's lane.
    fn count_bright_ramp_cells(buffer: &ratatui::buffer::Buffer, rect: Rect) -> usize {
        let bright: &[char] = &RAMP[2..];
        let mut count = 0;
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                if let Some(cell) = buffer.cell(ratatui::layout::Position::new(x, y)) {
                    let symbol = cell.symbol();
                    if bright.iter().any(|ramp| symbol.contains(*ramp)) {
                        count += 1;
                    }
                }
            }
        }
        count
    }

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

    /// Concatenates the symbols of the cells inside `rect`, so assertions
    /// can scope their scans to a single widget's area.
    fn rect_symbols(buffer: &ratatui::buffer::Buffer, rect: Rect) -> String {
        let mut out = String::new();
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                if let Some(cell) = buffer.cell(ratatui::layout::Position::new(x, y)) {
                    out.push_str(cell.symbol());
                }
            }
        }
        out
    }

    #[test]
    fn test_info_column_degrades_monotonically() {
        // Stacked D8 anatomy: the info block is as wide as the card, so the
        // meta line truncates instead of dropping wholesale.
        let long_name = "ORTOGRAFÍA ALARGADA";
        let meta = Some("7/9 sectores · meta 175 CPM");
        let rungs = [28u16, 22, 16, 10];
        let contents: Vec<[String; 3]> = rungs
            .map(|w| info_column_content(long_name, "◎", "DESTINO ACTUAL", "7/9", meta, w))
            .to_vec();

        // Line 3 is the progress·meta line: the leading progress digits
        // survive every rung, the trailing CPM detail drops first.
        for c in &contents {
            assert!(
                c[2].starts_with("7/9"),
                "progress must lead every rung: {:?}",
                c[2]
            );
        }
        assert!(
            contents[0][2].ends_with("CPM"),
            "full meta fits at rung 28: {:?}",
            contents[0][2]
        );
        for c in &contents[1..] {
            assert!(
                !c[2].contains("CPM"),
                "narrow rungs must drop the CPM meta: {:?}",
                c
            );
        }

        // ...then the planet name ellipsizes once it no longer fits.
        assert_eq!(contents[0][0], long_name, "name fits at rung 28 untouched");
        assert!(
            contents[2][0].ends_with('…'),
            "rung 16 must ellipsize the name: {:?}",
            contents[2][0]
        );
        assert!(
            contents[3][0].ends_with('…'),
            "rung 10 must ellipsize the name: {:?}",
            contents[3][0]
        );

        // Glyph + badge + progress NEVER drop at any rung (content-wise).
        for c in &contents {
            let joined = c.join("\n");
            assert!(
                joined.contains('◎'),
                "glyph must survive every rung: {joined}"
            );
            assert!(
                joined.contains("DESTINO ACTUAL"),
                "badge must survive every rung: {joined}"
            );
            assert!(
                joined.contains("7/9"),
                "progress must survive every rung: {joined}"
            );
        }

        // The amount of shown detail never grows as the column shrinks.
        let detail: Vec<usize> = contents
            .iter()
            .map(|c| {
                c.iter()
                    .map(|line| UnicodeWidthStr::width(line.as_str()))
                    .sum()
            })
            .collect();
        assert!(
            detail.windows(2).all(|w| w[0] >= w[1]),
            "detail must be monotonic non-increasing as width shrinks: {detail:?}"
        );
    }

    #[test]
    fn test_full_mode_planets_are_borderless_sprites() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
        let buffer = terminal.backend().buffer();

        let map = map_body_area(Rect::new(0, 0, 120, 40));
        let cards = planet_layout(map, app.selected_planet_index);
        let progresses = Curriculum::all_tier_progress(&app.user_progress);

        for (i, card) in cards.iter().enumerate() {
            if card.width == 0 || card.height == 0 {
                continue; // off-window planet: zero rect, nothing rendered
            }
            let symbols = rect_symbols(buffer, *card);
            for corner in ['╭', '╮', '╰', '╯'] {
                assert!(
                    !symbols.contains(corner),
                    "card {i} must not render a rounded border inside its rect:\n{symbols}"
                );
            }
            let tp = &progresses[i];
            let (glyph, _, _) = planet_style(tp.status);
            assert!(
                symbols.contains(glyph),
                "card {i} must show status glyph '{glyph}' in:\n{symbols}"
            );
            // D8 stacked anatomy: the info lines below the lane carry the
            // full badge text again (the card-wide info block replaced the
            // Batch-A 2-column side sliver that clipped it). The lane
            // itself paints the ray-cast sphere: ramp glyphs only.
            let lane = rect_symbols(buffer, sprite_lane(*card));
            assert!(
                contains_ramp_glyph(&lane),
                "card {i} lane must paint sphere ramp glyphs (·░▒▓█): '{lane}'"
            );
        }
    }

    #[test]
    fn mainmenu_selected_planet_visible_at_end_of_band() {
        // Camera follow: selecting the last planet pans the window so card 6
        // stays visible at the band's trailing edge, with its full stacked
        // anatomy — sprite lane on top, selection marker leading the info
        // rows BELOW the lane.
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 6;
        app.ship.snap_to(6);

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
        let buffer = terminal.backend().buffer();

        let map = map_body_area(Rect::new(0, 0, 120, 40));
        let card = planet_layout(map, 6)[6];
        assert!(
            card.width > 0 && card.height > 0,
            "selected end-of-band planet must be on screen: {card:?}"
        );
        let lane = rect_symbols(buffer, sprite_lane(card));
        assert!(
            lane.chars().any(|c| !c.is_whitespace()),
            "card 6 sprite lane must render: '{lane}'"
        );
        let info_top = card.y + sprite_lane(card).height;
        let marker_row = rect_symbols(buffer, Rect::new(card.x, info_top, card.width, 1));
        assert!(
            marker_row.contains('▶'),
            "selected card's marker+name row must render below the lane: '{marker_row}'"
        );
    }

    #[test]
    fn planets_render_in_horizontal_tier_order() {
        // The visible window renders tier cards left→right in tier order,
        // each as a stacked card: ray-cast sphere in the top lane rows,
        // status glyph in the info rows below the lane.
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;
        app.ship.snap_to(0);

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
        let buffer = terminal.backend().buffer();

        let map = map_body_area(Rect::new(0, 0, 120, 40));
        let cards = planet_layout(map, 0);
        let progresses = Curriculum::all_tier_progress(&app.user_progress);
        let visible: Vec<(usize, Rect)> = cards
            .iter()
            .cloned()
            .enumerate()
            .filter(|(_, r)| r.width > 0 && r.height > 0)
            .collect();
        assert!(
            visible.len() >= 2,
            "expected a multi-card window: {visible:?}"
        );
        for w in visible.windows(2) {
            assert!(
                w[1].1.x > w[0].1.x,
                "cards must render in tier order left→right: {visible:?}"
            );
        }
        for (i, card) in &visible {
            let lane = rect_symbols(buffer, sprite_lane(*card));
            assert!(
                lane.chars().any(|c| !c.is_whitespace()),
                "tier {i} sprite lane must render: '{lane}'"
            );
            let info_below_lane = Rect::new(
                card.x,
                card.y + sprite_lane(*card).height,
                card.width,
                card.height.saturating_sub(sprite_lane(*card).height),
            );
            let info = rect_symbols(buffer, info_below_lane);
            let (glyph, _, _) = planet_style(progresses[*i].status);
            assert!(
                info.contains(glyph),
                "tier {i} status glyph must sit in the info rows below the lane: '{info}'"
            );
        }
    }

    #[test]
    fn ship_renders_in_top_gutter_row() {
        // The parked ship flies in the top gutter rows, above the card band:
        // no ship glyph may dip into the cards' sprite lanes while idle.
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;
        app.ship.snap_to(0);

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
        let buffer = terminal.backend().buffer();

        let map = map_body_area(Rect::new(0, 0, 120, 40));
        let gutter = ship_gutter(map);
        let mut ship_cells = 0usize;
        // █ is also ramp step 4 of the sphere lanes below the gutter, so
        // ship detection matches the ship-exclusive ◄/► glyphs only.
        for y in map.y..map.y + map.height {
            for x in map.x..map.x + map.width {
                if let Some(cell) = buffer.cell(ratatui::layout::Position::new(x, y))
                    && matches!(cell.symbol(), "◄" | "►")
                {
                    assert!(
                        y >= gutter.y && y < gutter.y + gutter.height,
                        "parked ship must stay inside the top gutter row (y={y})"
                    );
                    ship_cells += 1;
                }
            }
        }
        assert!(ship_cells > 0, "parked ship sprite must render");
    }

    #[test]
    fn mainmenu_map_body_shorter_than_thirteen_rows_falls_back_to_compact() {
        // Full-window heights of 18–21 leave a 9–12 row map body (the
        // 6+3 header/footer chrome reservation). Those used to reach Full
        // mode and must now fall back to Compact with the taller lane.
        // The user-visible tell is the ship sprite: `render_ship_sprite`
        // runs in Full mode only.
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;
        app.ship.snap_to(0);

        // 100×21 → map body 62×12: below the Full threshold → Compact.
        let compact_map = map_body_area(Rect::new(0, 0, 100, 21));
        assert_eq!(
            compact_map.height, 12,
            "fixture premise: body height after the 6+3 chrome reservation"
        );
        assert_eq!(map_mode(compact_map), MapMode::Compact);
        assert!(
            !render_to_string(&app, 100, 21).contains("◄███►"),
            "no ship lane below the 13-row Full threshold"
        );

        // 100×22 → map body 62×13: one row above the threshold → Full,
        // with exactly one card band (3 gutter + 10 card rows) fitting.
        let full_map = map_body_area(Rect::new(0, 0, 100, 22));
        assert_eq!(full_map.height, 13, "fixture premise: threshold + 1");
        assert_eq!(map_mode(full_map), MapMode::Full);
        assert!(
            render_to_string(&app, 100, 22).contains("◄███►"),
            "Full mode renders the ship gutter at threshold + 1"
        );
    }

    #[test]
    fn full_mode_sphere_spans_the_tall_lane() {
        // The ray-cast disc is 7×7 inside the 12-wide lane: centered
        // horizontally (glyph offset 2) and spanning the lane's full
        // height, so the sphere silhouette reaches the lane's top and
        // bottom rows alike — no blank filler rows below the sphere.
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;
        app.ship.snap_to(0);

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
        let buffer = terminal.backend().buffer();

        let map = map_body_area(Rect::new(0, 0, 120, 40));
        for (i, card) in planet_layout(map, 0).iter().enumerate() {
            if card.width == 0 || card.height == 0 {
                continue;
            }
            let lane = sprite_lane(*card);
            assert_eq!(lane.height, 7, "card {i} lane must be the tall sphere lane");
            for (row, label) in [(0u16, "top"), (lane.height - 1, "bottom")] {
                let row_symbols =
                    rect_symbols(buffer, Rect::new(lane.x, lane.y + row, lane.width, 1));
                assert!(
                    contains_ramp_glyph(&row_symbols),
                    "card {i} sphere must reach the lane's {label} row: '{row_symbols}'"
                );
            }
        }
    }

    #[test]
    fn no_border_chrome_inside_planet_cards() {
        // Cards are chrome-free across their FULL stacked extent: no
        // retro-block corner glyphs inside any visible card, and the info
        // rows below each sprite lane must carry text. The ray-cast sphere
        // paints only ramp glyphs, so double-line box chrome (the legacy
        // tier-4 sprite art) is banned alongside the rounded-block chrome.
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;
        app.ship.snap_to(0);

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
        let buffer = terminal.backend().buffer();

        let map = map_body_area(Rect::new(0, 0, 120, 40));
        for (i, card) in planet_layout(map, 0).iter().enumerate() {
            if card.width == 0 || card.height == 0 {
                continue;
            }
            let symbols = rect_symbols(buffer, *card);
            for corner in ['╭', '╮', '╰', '╯', '╔', '╗', '╚', '╝', '║'] {
                assert!(
                    !symbols.contains(corner),
                    "card {i} must not render border chrome inside its rect:\n{symbols}"
                );
            }
            let info_below_lane = Rect::new(
                card.x,
                card.y + sprite_lane(*card).height,
                card.width,
                card.height.saturating_sub(sprite_lane(*card).height),
            );
            let info = rect_symbols(buffer, info_below_lane);
            assert!(
                info.chars().any(|c| !c.is_whitespace()),
                "card {i} info rows below the lane must render text: '{info}'"
            );
        }
    }

    #[test]
    fn mainmenu_planet_lane_rotates_over_time() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;
        // Slice-1 occlusion constraint: a ship DOCKED on a tier-4 card
        // covers ~7 left columns of that sprite lane (SHIP_FRAME_WIDTH), so
        // frame-diff tests must either diff only visible columns or park
        // the ship elsewhere. Here the ship parks idle in the top gutter
        // over tier 0, so every lane's differing columns stay uncovered.
        app.ship.snap_to(0);

        let draw = |app: &App| {
            let backend = TestBackend::new(120, 40);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|f| render(f, app)).unwrap();
            terminal.backend().buffer().clone()
        };

        let before = draw(&app);
        // 1600ms is exactly two legacy idle sprite periods — it would park
        // the old two-frame sprite back on frame 0 — while every tier's
        // per-config rotation period (2400–8000ms) lands on a distinct
        // nonzero phase, so a rotating sphere must move at least one lane
        // cell per card.
        app.ship.advance(Duration::from_millis(1600));
        let after = draw(&app);

        let map = map_body_area(Rect::new(0, 0, 120, 40));
        for (i, card) in planet_layout(map, 0).iter().enumerate() {
            if card.width == 0 || card.height == 0 {
                continue; // off-window planet: no sprite lane on screen
            }
            if matches!(i, 3 | 4) {
                // GasBands tiers (SÍMBOLOS, CADENCIA) are pure latitude
                // belts (`gas_bands_step` takes no longitude), so their
                // discs are longitude-invariant and rotation is invisible
                // at map-card fidelity by construction. The mission clock
                // itself is global: it provably drives every lon-dependent
                // archetype below. Design gap recorded for sdd-verify.
                continue;
            }
            let a = rect_symbols(&before, sprite_lane(*card));
            let b = rect_symbols(&after, sprite_lane(*card));
            assert_ne!(
                a, b,
                "tier {i} sphere lane must rotate after 1600ms of mission time"
            );
        }
    }

    #[test]
    fn reduced_motion_pins_planet_sphere_at_first_phase() {
        // Inverse of `mainmenu_planet_lane_rotates_over_time`: the same
        // time advance must leave every lane byte-identical (and still
        // drawn — a blank lane would trivially pass) when reduced motion
        // pins the sphere at the t=0 rotation phase.
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;
        app.ship.snap_to(0); // park idle in the gutter, lanes uncovered
        app.reduced_motion = true;

        let draw = |app: &App| {
            let backend = TestBackend::new(120, 40);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|f| render(f, app)).unwrap();
            terminal.backend().buffer().clone()
        };

        let before = draw(&app);
        // 1600ms mirrors the rotation test; under reduced motion (D10) the
        // amount is arbitrary — every advance must leave the spheres pinned
        // at the t=0 phase.
        app.ship.advance(Duration::from_millis(1600));
        let after = draw(&app);

        let map = map_body_area(Rect::new(0, 0, 120, 40));
        for (i, card) in planet_layout(map, 0).iter().enumerate() {
            if card.width == 0 || card.height == 0 {
                continue; // off-window planet: no sprite lane on screen
            }
            let frozen = rect_symbols(&before, sprite_lane(*card));
            assert!(
                contains_ramp_glyph(&frozen),
                "tier {i} frozen sphere must still draw ramp glyphs: '{frozen}'"
            );
            assert_eq!(
                frozen,
                rect_symbols(&after, sprite_lane(*card)),
                "tier {i} sphere lane must stay frozen at the t=0 phase"
            );
        }
    }

    #[test]
    fn reduced_motion_ship_flicker_pinned_to_frame_zero() {
        // +120ms flips ONLY the thruster flicker in the gutter (the spheres
        // are D10-frozen in the reduced render). Under reduced motion the
        // gutter must stay identical; a full-mode companion renders the
        // same delta and MUST flicker, proving the setup exercises the
        // sprite clock (no trivially-green equality).
        let build = |reduced: bool| {
            let mut app = App::new();
            app.user_progress = UserProgress::default();
            app.selected_planet_index = 0;
            app.ship.snap_to(0);
            app.reduced_motion = reduced;
            app
        };
        let gutter_symbols = |app: &App| {
            let backend = TestBackend::new(120, 40);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|f| render(f, app)).unwrap();
            let map = map_body_area(Rect::new(0, 0, 120, 40));
            rect_symbols(terminal.backend().buffer(), ship_gutter(map))
        };

        // Full mode: the thruster flicker visibly changes the gutter.
        let mut full = build(false);
        let before = gutter_symbols(&full);
        full.ship.advance(Duration::from_millis(120));
        let after = gutter_symbols(&full);
        assert_ne!(before, after, "full mode must flicker at +120ms");

        // Reduced mode: same delta, frozen gutter.
        let mut reduced = build(true);
        let pinned_before = gutter_symbols(&reduced);
        reduced.ship.advance(Duration::from_millis(120));
        let pinned_after = gutter_symbols(&reduced);
        assert_eq!(
            pinned_before, pinned_after,
            "reduced motion must pin the ship sprite to frame 0"
        );
    }

    #[test]
    fn reduced_motion_buffers_identical_over_time() {
        // Whole-screen freeze: at t=0 vs t=+2s a full-mode render rotates
        // every sphere (2s is a nonzero fraction of each tier's
        // 2400–8000ms period) and flips the thruster flicker (~16 cycles);
        // reduced motion must render the exact same buffer, proving no
        // ambient animation leaks through anywhere.
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;
        app.ship.snap_to(0);
        app.reduced_motion = true;

        let draw = |app: &App| {
            let backend = TestBackend::new(120, 40);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|f| render(f, app)).unwrap();
            terminal.backend().buffer().clone()
        };

        let before = draw(&app);
        app.ship.advance(Duration::from_secs(2));
        let after = draw(&app);

        assert_eq!(before, after, "buffers must be identical across 2s");
    }

    #[test]
    fn mainmenu_renders_sphere_lane_for_every_tier() {
        // The band window pans with the selection, so driving `selected`
        // through every tier proves each tier's sphere lane renders ramp
        // glyphs (each tier's own palette + surface archetype).
        let map = map_body_area(Rect::new(0, 0, 120, 40));
        for sel in 0..crate::tui::planet_layout::PLANET_COUNT {
            let mut app = App::new();
            app.user_progress = UserProgress::default();
            app.selected_planet_index = sel;
            let backend = TestBackend::new(120, 40);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|f| render(f, &app)).unwrap();
            let buffer = terminal.backend().buffer();

            let card = planet_layout(map, sel)[sel];
            assert!(
                card.width > 0 && card.height > 0,
                "selected tier {sel} must be inside the camera window"
            );
            let lane = rect_symbols(buffer, sprite_lane(card));
            assert!(
                contains_ramp_glyph(&lane),
                "tier {sel} sphere lane must render ramp glyphs: '{lane}'"
            );
        }
    }

    #[test]
    fn test_selected_planet_card_shows_marker_and_bold() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
        let buffer = terminal.backend().buffer();
        let map = map_body_area(Rect::new(0, 0, 120, 40));
        let cards = planet_layout(map, 0);
        let sel_info = info_column(cards[0]);
        assert!(
            sel_info.width > 0,
            "selected card must keep an info column sliver"
        );
        // Batch-A note: the narrow band card ellipsizes the name, so the
        // marker is asserted cell-wise rather than as a full string.
        let marker = buffer
            .cell(ratatui::layout::Position::new(sel_info.x, sel_info.y))
            .expect("selected info column origin must carry the marker cell");
        assert!(
            marker.symbol() == "▶" && marker.style().add_modifier.contains(Modifier::BOLD),
            "selected row's marker/name must be '▶' and BOLD: {:?}",
            marker.style()
        );
        for (i, card) in cards.iter().enumerate().skip(1) {
            if card.width == 0 || card.height == 0 {
                continue; // off-window planet renders nothing
            }
            let symbols = rect_symbols(buffer, info_column(*card));
            assert!(
                !symbols.contains('▶'),
                "unselected card {i} must not carry the marker:\n{symbols}"
            );
        }
    }

    #[test]
    fn test_render_ship_in_lane_during_travel() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;
        app.move_planet_right(); // Traveling toward planet 1, still mid-flight

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
        let buffer = terminal.backend().buffer();

        let map = map_body_area(Rect::new(0, 0, 120, 40));
        let lane_bottom = map.y + SHIP_GUTTER_HEIGHT;
        let mut ship_min_x: Option<u16> = None;
        // █ doubles as the sphere lanes' top ramp step, so the travelling
        // ship is detected by its exclusive ◄/► bow/stern glyphs.
        for y in map.y..map.y + map.height {
            for x in map.x..map.x + map.width {
                if let Some(cell) = buffer.cell(ratatui::layout::Position::new(x, y))
                    && matches!(cell.symbol(), "◄" | "►")
                {
                    assert!(
                        y < lane_bottom,
                        "travelling ship must fly inside the top gutter row (y={y})"
                    );
                    ship_min_x = Some(ship_min_x.map_or(x, |m: u16| m.min(x)));
                }
            }
        }
        let ship_min_x = ship_min_x.expect("travelling ship must render its sprite");
        assert!(
            ship_min_x >= map.x,
            "ship glyphs must stay inside the map body (x >= {}), found min x {ship_min_x}",
            map.x
        );

        // WARP_TRAIL streaks (═ is exclusive to the trail) flicker behind the
        // ship, pointing back toward the gutter, during the whole flight.
        let rendered = render_to_string(&app, 120, 40);
        assert!(
            rendered.contains('═'),
            "WARP_TRAIL must be visible during travel:\n{rendered}"
        );
    }

    #[test]
    fn test_planet_at_never_resolves_under_ship() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier4NumbersAndSymbols.index();
        app.ship = crate::tui::animation::ShipAnimation::new(app.selected_planet_index);
        app.confirm_planet();
        // Mid-dock: advance exactly to the cubic-out t where progress() ==
        // 0.5, so the ship's lane x is the midpoint of the ~6-cell dock.
        let half_t = crate::tui::animation::DESCEND.mul_f32(1.0 - (0.5f64).cbrt() as f32);
        app.advance_animation(half_t);

        let map = map_body_area(Rect::new(0, 0, 120, 40));
        let cards = planet_layout(map, app.selected_planet_index);
        let i = app.selected_planet_index;
        // Mid-dock: the ship dips from the flight lane onto card i's top
        // row (the sprite lane's first row). A point on that top row —
        // directly under the docking ship — must still resolve to card i:
        // cards drive hit-testing, the ship sprite never does.
        let lane = sprite_lane(cards[i]);
        let under_ship = ratatui::layout::Position::new(lane.x + lane.width / 2, cards[i].y);
        assert_eq!(
            planet_at(map, under_ship, i),
            Some(i),
            "hit-testing must resolve by card rect, never by the ship sprite"
        );
    }

    #[test]
    fn test_ship_sprite_parks_over_selected_card_full_mode() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;
        // Real flows always move the ship with the selection; App::new()
        // loads the developer's saved progress, so snap explicitly.
        app.ship.snap_to(app.selected_planet_index);

        // 120x40 is the smallest round size that actually reaches Full mode:
        // the map body is 62% of window width and the derived MIN_FULL_HEIGHT
        // is checked against `chunks[1].height` (window height minus the
        // 6-row header and 3-row footer).
        let rendered_full = render_to_string(&app, 120, 40);
        assert!(
            rendered_full.contains("◄███►"),
            "expected ship sprite in Full mode:\n{rendered_full}"
        );

        // The horizontal band parks the ship over the selected card's
        // columns (its lerped center), not at an arbitrary map column.
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
        let buffer = terminal.backend().buffer();
        let map = map_body_area(Rect::new(0, 0, 120, 40));
        let card = planet_layout(map, app.selected_planet_index)[app.selected_planet_index];
        let mut leftmost: Option<u16> = None;
        'scan: for x in map.x..map.x + map.width {
            for y in map.y..map.y + map.height {
                if matches!(
                    buffer
                        .cell(ratatui::layout::Position::new(x, y))
                        .map(|c| c.symbol()),
                    Some("◄" | "►")
                ) {
                    leftmost = Some(x);
                    break 'scan;
                }
            }
        }
        let leftmost = leftmost.expect("ship sprite must render in Full mode");
        assert!(
            leftmost >= card.x && leftmost < card.x + PLANET_CARD_WIDTH,
            "ship must park over the selected card ({}..{}), found leftmost glyph at x {leftmost}",
            card.x,
            card.x + PLANET_CARD_WIDTH
        );

        let rendered_compact = render_to_string(&app, 40, 12);
        assert!(
            !rendered_compact.contains("◄███►"),
            "compact mode must not render the ship sprite:\n{rendered_compact}"
        );
    }

    #[test]
    fn test_render_dock_visible_before_flip() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        // Real flows always move the ship with the selection (App::new()
        // loads the developer's saved progress); park it on the target so
        // the dock below happens at the selected card, like Enter does.
        app.ship.snap_to(app.selected_planet_index);
        app.confirm_planet();
        // Advance to the cubic-out midpoint of DESCEND: dock_depth == 0.5
        // puts the ship's lane y halfway between flight lane and card top.
        let half_t = crate::tui::animation::DESCEND.mul_f32(1.0 - (0.5f64).cbrt() as f32);
        app.advance_animation(half_t);

        assert_eq!(
            app.current_view,
            CurrentView::MainMenu,
            "the dock itself must never flip the view (only a second Enter opens)"
        );

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
        let buffer = terminal.backend().buffer();

        // MainMenu keeps rendering, docking ship included, for the whole
        // descent; the docked state it lands in is the map's to keep.
        let rendered = render_to_string(&app, 120, 40);
        assert!(
            rendered.contains("◄███►"),
            "docking ship must stay visible pre-flip:\n{rendered}"
        );
        assert!(
            rendered.contains('◎'),
            "MainMenu content must render pre-flip:\n{rendered}"
        );

        // The dock is a real vertical move onto the planet's card: the ship's
        // column stays pinned over the target card's span for the whole
        // descent while its rows dip onto the card's top row.
        let mut ship_col: Option<u16> = None;
        'scan: for y in 0..buffer.area().height {
            for x in 0..buffer.area().width {
                if matches!(
                    buffer
                        .cell(ratatui::layout::Position::new(x, y))
                        .map(|c| c.symbol()),
                    Some("◄")
                ) {
                    ship_col = Some(x);
                    break 'scan;
                }
            }
        }
        let ship_col = ship_col.expect("docking ship must render its leftmost glyph");
        let card = planet_layout(
            map_body_area(Rect::new(0, 0, 120, 40)),
            app.selected_planet_index,
        )[app.selected_planet_index];
        assert!(
            ship_col >= card.x && ship_col < card.x + card.width,
            "docking ship must stay over the target card ({}..{}), found leftmost glyph at x {ship_col}",
            card.x,
            card.x + card.width
        );
    }

    #[test]
    fn docked_ship_pinned_to_lane_core() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;
        app.ship = crate::tui::animation::ShipAnimation::new(0);
        app.confirm_planet();
        app.finish_ship_animation();
        assert!(app.is_docked());

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
        let buffer = terminal.backend().buffer();

        let map = map_body_area(Rect::new(0, 0, 120, 40));
        let card = planet_layout(map, app.selected_planet_index)[app.selected_planet_index];
        let gutter = ship_gutter(map);
        let gutter_core_y = gutter.y + gutter.height / 2;

        // Collect the rows carrying the ship's ◄/► markers (exclusive to
        // the ship sprite; sphere lanes never use them).
        let mut ship_rows: Vec<u16> = Vec::new();
        for y in 0..buffer.area().height {
            for x in 0..buffer.area().width {
                if matches!(
                    buffer
                        .cell(ratatui::layout::Position::new(x, y))
                        .map(|c| c.symbol()),
                    Some("◄" | "►")
                ) {
                    ship_rows.push(y);
                }
            }
        }
        assert!(!ship_rows.is_empty(), "docked ship must render its sprite");

        // D9: the docked ship stays pinned into the target card's sprite
        // lane (its marker row is the card's top row), instead of popping
        // back out to the gutter's core row the way an idle ship would.
        assert!(
            ship_rows.contains(&card.y),
            "docked ship's marker row must be the card's top row {}, found {ship_rows:?}",
            card.y
        );
        assert!(
            !ship_rows.contains(&gutter_core_y),
            "docked ship must not sit in the gutter core row {gutter_core_y}"
        );
    }

    #[test]
    fn docked_planet_lane_keeps_painting_the_sphere() {
        // The legacy docked sprite breathed at a slower frame period (D5);
        // the ray-cast sphere replaces that mechanism with each tier's
        // fixed config rotation period. What must survive docking is the
        // sphere itself: the selected card's lane keeps painting ramp
        // glyphs underneath the docked ship (the ship's 3-row sprite at
        // most covers rows card.y-2..=card.y+1, so lane rows from y+2
        // down are always ship-free).
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;
        app.ship = crate::tui::animation::ShipAnimation::new(0);
        app.confirm_planet();
        app.finish_ship_animation();
        assert!(app.is_docked());

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
        let buffer = terminal.backend().buffer();

        let map = map_body_area(Rect::new(0, 0, 120, 40));
        let card = planet_layout(map, app.selected_planet_index)[app.selected_planet_index];
        let lane = sprite_lane(card);
        assert_eq!(lane.height, 7, "docked card keeps the tall sphere lane");
        let below_ship = rect_symbols(
            buffer,
            Rect::new(lane.x, lane.y + 2, lane.width, lane.height - 2),
        );
        assert!(
            contains_ramp_glyph(&below_ship),
            "docked planet must keep painting its sphere below the ship: '{below_ship}'"
        );
    }

    #[test]
    fn test_80x24_terminal_renders_full_mode() {
        // 80x24 is a very common default terminal size; with the Full-mode
        // threshold (46x6 on the map body), it must render the horizontal
        // band — ship sprite in the top lane plus planet sphere lanes —
        // not the cramped Compact layout.
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;

        let rendered = render_to_string(&app, 80, 24);

        assert!(
            rendered.contains("◄███►"),
            "expected ship sprite in:\n{rendered}"
        );
        assert!(
            rendered.contains('◎'),
            "expected current-planet status glyph in:\n{rendered}"
        );

        // Planet lanes: each visible card's lane paints ray-cast sphere
        // ramp glyphs (80×24 fits ~3 cards).
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
        let buffer = terminal.backend().buffer();
        let map = map_body_area(Rect::new(0, 0, 80, 24));
        for (i, card) in planet_layout(map, 0).iter().enumerate() {
            if card.width == 0 || card.height == 0 {
                continue;
            }
            let lane = rect_symbols(buffer, sprite_lane(*card));
            assert!(
                contains_ramp_glyph(&lane),
                "80x24 card {i} lane must paint sphere ramp glyphs: '{lane}'"
            );
        }
    }

    #[test]
    fn observatory_glyphs_stay_out_of_map_and_summary_renders() {
        // Dock-only invariant (approval guard): the observatory's ring,
        // moon, and HUD glyphs may never leak into map-only or summary
        // renders. Green before AND after the observatory view wiring,
        // so the wiring can never silently regress it.
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;

        let map = render_to_string(&app, 100, 24);
        assert!(!map.contains('≡'), "ring glyphs must stay dock-only");
        assert!(!map.contains('☾'), "moon glyphs must stay dock-only");
        assert!(
            !map.contains("OBSERVATORIO"),
            "HUD labels must stay dock-only"
        );

        app.start_practice(Curriculum::all_lessons()[0].clone());
        app.current_view = CurrentView::Summary;
        let summary = render_to_string(&app, 100, 24);
        assert!(!summary.contains('≡'), "ring glyphs must stay dock-only");
        assert!(!summary.contains('☾'), "moon glyphs must stay dock-only");
        assert!(
            !summary.contains("OBSERVATORIO"),
            "HUD labels must stay dock-only"
        );
    }

    #[test]
    fn observatory_view_renders_the_dock_frame() {
        // Full wiring: an App parked on CurrentView::Observatory must
        // render the docked planet — rings, moon, and HUD — through the
        // shared render() dispatch.
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;
        app.current_view = CurrentView::Observatory;

        let rendered = render_to_string(&app, 100, 24);

        assert!(rendered.contains('≡'), "dock frame must show ring glyphs");
        assert!(rendered.contains('☾'), "dock frame must show the moon");
        assert!(
            rendered.contains("OBSERVATORIO"),
            "dock frame must show the HUD title"
        );
        // Telemetry reflects the selected planet: tier 1 CIMIENTOS.
        assert!(
            rendered.contains("CIMIENTOS"),
            "HUD must name the selected planet"
        );
    }

    #[test]
    fn finished_session_relights_passed_tier_card_conquered() {
        // Integration (three-step flow + status-as-lighting): pass a
        // lesson through the real engine, return to the galaxy map, and
        // the passed tier's card must render Conquered lighting — more
        // day-lit ramp cells than its pre-session Current look, and
        // strictly brighter than any still-Unexplored tier's night card.
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        app.ship.snap_to(app.selected_planet_index);
        assert_eq!(
            Curriculum::all_tier_progress(&app.user_progress)[0].status,
            PlanetStatus::Current,
            "fixture premise: Tier1 starts as the Current destination"
        );

        let map = map_body_area(Rect::new(0, 0, 120, 40));
        let tier1_lane = sprite_lane(planet_layout(map, app.selected_planet_index)[0]);
        let tier3_lane = sprite_lane(planet_layout(map, app.selected_planet_index)[2]);

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
        let bright_before = count_bright_ramp_cells(terminal.backend().buffer(), tier1_lane);
        assert!(
            bright_before >= 1,
            "fixture premise: the Current dawn crescent must already light some day cells ({bright_before})"
        );

        // Pass Tier1's first lesson through the real flow: three-step
        // Enter into the lessons, then a flawless run through the engine.
        open_lessons_via_three_step_enter(&mut app);
        let lesson = app
            .selected_lesson()
            .expect("a lesson must be selected")
            .clone();
        app.start_practice(lesson);
        let text = app.current_engine.as_ref().unwrap().lesson.text.clone();
        for ch in text.chars() {
            app.handle_key_input(ch);
        }
        assert!(
            app.last_session_passed,
            "fixture premise: the flawless run must pass"
        );
        app.return_from_session(); // Summary -> PlanetLessons (lesson row)
        app.leave_planet_lessons(); // -> MainMenu: the map return

        assert_eq!(
            Curriculum::all_tier_progress(&app.user_progress)[0].status,
            PlanetStatus::Conquered,
            "passing the lesson must conquer Tier1 in the derived statuses"
        );

        terminal.draw(|f| render(f, &app)).unwrap();
        let buffer = terminal.backend().buffer();
        let bright_after = count_bright_ramp_cells(buffer, tier1_lane);
        let tier3_bright = count_bright_ramp_cells(buffer, tier3_lane);

        assert!(
            bright_after > bright_before,
            "the Conquered relight must add day-lit cells to the passed card (before {bright_before}, after {bright_after})"
        );
        assert!(
            bright_after > tier3_bright,
            "the conquered card must out-shine a still-Unexplored night card ({bright_after} vs {tier3_bright})"
        );
        assert!(
            contains_ramp_glyph(&rect_symbols(buffer, tier1_lane)),
            "the relit card must keep painting its sphere"
        );
    }

    #[test]
    fn test_full_mode_card_shows_progress_cpm_and_badge() {
        // D8 stacked anatomy: the card-wide info lines below the sprite
        // lane restore the per-tier text — status badge and progress·meta
        // — alongside the status glyphs across the visible window.
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;

        let rendered = render_to_string(&app, 120, 40);

        assert!(
            rendered.contains('◎'),
            "expected current-tier glyph in:\n{rendered}"
        );
        assert!(
            rendered.contains('○'),
            "expected unexplored glyph in:\n{rendered}"
        );
        assert!(
            rendered.contains("SIN EXPLORAR"),
            "expected unexplored badge text (fits the 14-col card) in:\n{rendered}"
        );
        assert!(
            rendered.contains("sectores"),
            "expected progress·meta line in:\n{rendered}"
        );
        assert!(
            rendered.contains('/'),
            "expected progress digits in:\n{rendered}"
        );
    }

    #[test]
    fn test_100x34_shows_cimientos_and_sin_explorar_badges() {
        // Deterministic progress: Tier1 unlocked, nothing completed yet, so
        // the galaxy map always shows the current destination and every
        // other planet as unexplored regardless of the player's real saved
        // progress file. The D8 card-wide info lines carry the badge text.
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;

        let rendered = render_to_string(&app, 100, 34);

        assert!(
            rendered.contains('◎'),
            "expected current-planet glyph in:\n{rendered}"
        );
        assert!(
            rendered.contains("SIN EXPLORAR"),
            "expected unexplored badge text in:\n{rendered}"
        );
    }

    #[test]
    fn test_40x12_compact_shows_7_columns_no_sprite_no_panic() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;

        let rendered = render_to_string(&app, 40, 12);

        // Compact = seven equal one-row columns; each shows at least its
        // status glyph (names clip to the column width).
        let map = map_body_area(Rect::new(0, 0, 40, 12));
        let progresses = Curriculum::all_tier_progress(&app.user_progress);
        let backend = TestBackend::new(40, 12);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
        let buffer = terminal.backend().buffer();
        for (i, card) in planet_layout(map, app.selected_planet_index)
            .iter()
            .enumerate()
        {
            let (glyph, _, _) = planet_style(progresses[i].status);
            let symbols = rect_symbols(buffer, *card);
            assert!(
                symbols.contains(glyph),
                "compact column {i} must show status glyph '{glyph}': '{symbols}'"
            );
        }
        assert!(
            !rendered.contains('▲'),
            "compact mode must not render the ship sprite:\n{rendered}"
        );
        assert!(
            !rendered.contains('█'),
            "compact mode must not render the ship sprite:\n{rendered}"
        );
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
        assert_eq!(
            planet_style(PlanetStatus::Conquered),
            ("◉", Theme::SUCCESS, "CONQUISTADO")
        );
        assert_eq!(
            planet_style(PlanetStatus::Current),
            ("◎", Theme::PRIMARY, "DESTINO ACTUAL")
        );
        assert_eq!(
            planet_style(PlanetStatus::InProgress),
            ("◍", Theme::ACCENT, "EN CURSO")
        );
        assert_eq!(
            planet_style(PlanetStatus::Unexplored),
            ("○", Theme::MUTED, "SIN EXPLORAR")
        );
    }

    #[test]
    fn test_map_body_area_matches_render_main_menu_body_left_chunk() {
        // Approval test: reproduces the pre-extraction inline Layout math so
        // `map_body_area` is proven equivalent, not just plausible.
        let area = Rect::new(0, 0, 120, 40);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),
                Constraint::Min(12),
                Constraint::Length(3),
            ])
            .split(area);
        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
            .split(chunks[1]);

        assert_eq!(map_body_area(area), body_chunks[0]);
    }

    #[test]
    fn test_lesson_list_area_matches_map_body_area() {
        // `render_planet_lessons` uses the identical outer chunking as the
        // galaxy map, so the list block occupies the same left-panel rect.
        let area = Rect::new(0, 0, 100, 34);
        assert_eq!(lesson_list_area(area), map_body_area(area));
    }

    /// Three-step Enter through the public API (D2): confirm starts the
    /// descend, finish docks it, the second confirm opens the
    /// observatory, and the observatory's Enter opens the lessons.
    /// Leaves the app in PlanetLessons — replaces the old two-step
    /// setup idiom.
    fn open_lessons_via_three_step_enter(app: &mut App) {
        app.confirm_planet();
        app.finish_ship_animation();
        app.confirm_planet();
        assert_eq!(app.current_view, CurrentView::Observatory);
        app.open_planet_lessons();
        assert_eq!(app.current_view, CurrentView::PlanetLessons);
    }

    #[test]
    fn test_planet_lessons_view_lists_only_selected_tier() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        open_lessons_via_three_step_enter(&mut app);

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

        assert!(
            rendered.contains(&tier1_title),
            "expected Tier1 lesson '{tier1_title}' in:\n{rendered}"
        );
        assert!(
            rendered.contains(Tier::Tier1Foundation.planet_name()),
            "expected Tier1 planet name in:\n{rendered}"
        );
        assert!(
            !rendered.contains(&tier2_title),
            "must not list Tier2 lesson '{tier2_title}' while on Tier1:\n{rendered}"
        );
    }

    #[test]
    fn test_planet_lessons_view_1x1_no_panic() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        open_lessons_via_three_step_enter(&mut app);

        let rendered = render_to_string(&app, 1, 1);
        assert_eq!(rendered.chars().count(), 1);
    }

    #[test]
    fn test_planet_lessons_view_shows_section_header() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        open_lessons_via_three_step_enter(&mut app);

        let section_title = Curriculum::all_sections()
            .into_iter()
            .find(|s| s.tier == Tier::Tier1Foundation)
            .expect("fixture needs a Tier1 section")
            .title;

        let rendered = render_to_string(&app, 100, 34);

        assert!(
            rendered.contains(&section_title),
            "expected section header '{section_title}' in:\n{rendered}"
        );
    }

    /// Builds a minimal, arbitrary [`SessionRecord`] for `CurrentView::History`
    /// rendering tests. Mirrors the fixture pattern already used in
    /// `storage::repository` and `core::model`'s own test modules.
    fn make_session_record(kind: SessionKind, completed_at: u64) -> SessionRecord {
        SessionRecord {
            completed_at,
            duration_secs: 42,
            summary: SessionSummary {
                cpm: 180.0,
                raw_wpm: 36.0,
                net_wpm: 34.0,
                accuracy: 96.5,
                consistency: 90.0,
                total_keystrokes: 120,
                correct_keystrokes: 116,
                error_count: 4,
            },
            kind,
        }
    }

    #[test]
    fn test_history_view_empty_renders_without_panic() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.current_view = CurrentView::History;

        // Must not panic; the assertion below is a secondary sanity check
        // that something was actually drawn (an empty-state message).
        let rendered = render_to_string(&app, 100, 30);
        assert!(!rendered.trim().is_empty());
    }

    #[test]
    fn test_history_view_mixed_kind_shows_both_kinds_and_excludes_out_of_scope_content() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.user_progress.sessions.push(make_session_record(
            SessionKind::Lesson {
                lesson_id: "t1-l1".to_string(),
                tier: Tier::Tier1Foundation,
                passed: true,
            },
            1_700_000_000,
        ));
        app.user_progress.sessions.push(make_session_record(
            SessionKind::Dictation {
                avg_reaction_time_ms: 300.0,
                min_reaction_time_ms: 150.0,
                max_reaction_time_ms: 500.0,
                total_words: 5,
                completed_words: 5,
            },
            1_700_000_100,
        ));
        app.current_view = CurrentView::History;

        let rendered = render_to_string(&app, 120, 30);

        assert!(
            rendered.contains("LECCIÓN"),
            "expected a Lesson-kind entry in:\n{rendered}"
        );
        assert!(
            rendered.contains("DICTADO"),
            "expected a Dictation-kind entry in:\n{rendered}"
        );

        // Spec rev 2's three negative scenarios (history-view content scope):
        // no confusion/substitution matrix, no per-finger latency breakdown,
        // no net-WPM figure, no export/import control. This data is stored
        // (SessionSummary::net_wpm, SessionBucket, etc.) but never surfaced
        // by this view.
        let lower = rendered.to_lowercase();
        assert!(
            !lower.contains("confus"),
            "must not render a confusion matrix:\n{rendered}"
        );
        assert!(
            !lower.contains("sustituci"),
            "must not render a substitution matrix:\n{rendered}"
        );
        assert!(
            !lower.contains("matriz"),
            "must not render any matrix widget:\n{rendered}"
        );
        assert!(
            !lower.contains("dedo"),
            "must not render a per-finger latency breakdown:\n{rendered}"
        );
        assert!(
            !lower.contains("wpm neto"),
            "must not render a net-WPM figure:\n{rendered}"
        );
        assert!(
            !lower.contains("exportar"),
            "must not render an export control:\n{rendered}"
        );
        assert!(
            !lower.contains("importar"),
            "must not render an import control:\n{rendered}"
        );
    }
}
