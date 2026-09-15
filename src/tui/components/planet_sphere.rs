//! Ray-cast planet map-card widget — the bridge between the pure planet
//! math in `crate::tui::planets` and the ratatui terminal buffer.
//!
//! Map cards (D8 stacked anatomy: the top lane of each Full-mode planet
//! card) render the shaded disc straight into the buffer: no `Paragraph`,
//! no `Vec<Line>`, zero heap allocation on the render path, truecolor
//! `Color::Rgb` only. Palette values are converted to terminal colors
//! here and nowhere else — this module is the only place planet math
//! meets ratatui. The observatory mode (a later slice) shares this file.

use std::time::Duration;

use crate::core::model::PlanetStatus;
use crate::tui::planets::{
    CELL_ASPECT, PLANET_CONFIGS, RAMP, Rgb, ShadedCell, disc_cols, rotation_phase, shade_disc,
};
use crate::tui::theme::Theme;

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;

/// Map-card disc row count: lanes are 7 rows tall, so the disc is
/// 7 rows — the design's map-card sphere (the observatory renders larger
/// discs). The disc's column count is derived from this via
/// [`disc_cols`] so the silhouette reads round on screen rather than as
/// a tall ellipse (terminal cells are ~2x taller than wide).
const MAP_CARD_DISC_ROWS: u16 = 7;

/// Stack backing for one map-card disc (`rows * disc_cols(rows)` cells)
/// — the render path allocates nothing on the heap.
const MAP_CARD_CELLS: usize = MAP_CARD_DISC_ROWS as usize * disc_cols(MAP_CARD_DISC_ROWS) as usize;

/// Minimum ramp step painted in the palette's primary (day) tone; dimmer
/// steps take the secondary (night) tone, so day glyphs always outshine
/// night glyphs and the night floor keeps unlit cells dark.
const DAY_STEP_MIN: u8 = 2;

/// Renders the planet of tier `tier_idx` (indexed like `PLANET_CONFIGS`)
/// as a map-card disc inside `lane`, lit for `status` and rotated by
/// elapsed mission time `t`; `reduced_motion` (D10) freezes the sphere at
/// the t=0 phase.
///
/// Panic-free by contract: every write goes through `cell_mut`, so
/// clipped or degenerate lanes simply paint less. Unknown tier indices
/// render nothing. Zero heap allocation: the shaded cells live on the
/// stack and the buffer is written in place.
pub fn render_map_card(
    buf: &mut Buffer,
    lane: Rect,
    tier_idx: usize,
    status: PlanetStatus,
    t: Duration,
    reduced_motion: bool,
) {
    let Some(cfg) = PLANET_CONFIGS.get(tier_idx) else {
        return;
    };
    let rows = lane.height.min(MAP_CARD_DISC_ROWS);
    let cols = disc_cols(rows).min(lane.width);
    let mut cells = [ShadedCell::BLANK; MAP_CARD_CELLS];
    let phase = rotation_phase(t, cfg.rotation_period, reduced_motion);
    shade_disc(cfg, status, phase, cols, rows, &mut cells);

    // The palette -> terminal-color bridge (`tone` below) is the only
    // place pure `planets::palette` values become ratatui colors.
    let palette = cfg.palette;
    let day = tone(palette.primary);
    let night = tone(palette.secondary);

    let offset_x = (lane.width - cols) / 2;
    let offset_y = (lane.height - rows) / 2;
    for row in 0..rows {
        for col in 0..cols {
            let Some(step) = cells[row as usize * cols as usize + col as usize].step else {
                continue;
            };
            let x = lane.x + offset_x + col;
            let y = lane.y + offset_y + row;
            // `cell_mut` is None outside the buffer: the clip guard and
            // the direct write in one panic-free step.
            if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                cell.set_char(RAMP[step as usize])
                    .set_fg(if step >= DAY_STEP_MIN { day } else { night });
            }
        }
    }
}

/// Converts a pure palette tone into a terminal truecolor value — the
/// single bridge from `planets::palette` to ratatui color.
fn tone(rgb: Rgb) -> Color {
    Color::Rgb(rgb.r, rgb.g, rgb.b)
}

/// One full revolution of the observatory moon around its planet.
const MOON_PERIOD: Duration = Duration::from_millis(12_000);

/// Observatory disc row-count clamp bounds (design: observatory
/// structure) — the docked view renders the sphere far larger than the
/// 7-row map cards. The disc's column count is derived from the row
/// count via [`disc_cols`] (terminal cells are ~2x taller than wide).
const OBS_DISC_MIN: u16 = 9;
const OBS_DISC_MAX: u16 = 21;

/// Stack backing for the largest observatory disc (`OBS_DISC_MAX *
/// disc_cols(OBS_DISC_MAX)` cells) — the render path allocates nothing
/// on the heap.
const OBS_DISC_CELLS: usize = OBS_DISC_MAX as usize * disc_cols(OBS_DISC_MAX) as usize;

/// Ring band geometry in horizontal cell units beyond the disc's
/// horizontal radius `Rx`: the band spans `+-(Rx + RING_SPAN *
/// CELL_ASPECT)` with the Cassini division gap sitting at
/// `Rx + CASSINI_OFFSET * CELL_ASPECT`. Both offsets are horizontal cell
/// counts, so they scale by [`CELL_ASPECT`] to keep the same on-screen
/// proportions as the disc itself.
const RING_SPAN: i32 = 4;
const CASSINI_OFFSET: i32 = 2;

/// Horizontal margin (in columns) reserved beyond the disc width for the
/// moon's orbit and the ring band's tips on both sides. The moon's orbit
/// (`MOON_ORBIT * CELL_ASPECT` past the horizontal radius) reaches
/// farther than the ring band (`RING_SPAN * CELL_ASPECT` past it), so it
/// is the binding reserve — sizing for it keeps the ring tips inside the
/// canvas too.
const HORIZONTAL_MARGIN: u16 = 2 * MOON_ORBIT as u16 * CELL_ASPECT;

/// The moon orbits at horizontal radius `Rx + MOON_ORBIT * CELL_ASPECT`
/// on a squashed ellipse (a tilted orbital plane) so it can hide behind
/// and cross in front of the disc; the vertical orbit radius stays
/// `(Ry + MOON_ORBIT) * MOON_SQUASH` — a vertical offset needs no
/// [`CELL_ASPECT`] scaling since it is already in row units.
const MOON_ORBIT: i32 = 6;
const MOON_SQUASH: f32 = 0.5;

/// Number of dotted orbit-trace samples around the ellipse.
const TRACE_SAMPLES: usize = 24;

/// Renders the docked-planet observatory into `area`: the large ray-cast
/// sphere with rings, a moon on a dotted orbit, and a telemetry HUD.
///
/// Contract (design: observatory structure): the body splits 62/38 into
/// a planet canvas and a HUD panel; the disc row count clamps to
/// `clamp(min(canvas.h, rows implied by canvas.w), 9, 21)` and the
/// column count is `disc_cols(rows)` so the disc reads round on screen;
/// the ring band spans `+-(Rx + 4*CELL_ASPECT)` columns around the
/// equator with a Cassini division gap, its west arc passing in front of
/// the disc and its east arc behind it; the moon orbits outside the disc
/// on a squashed ellipse, occluded on the far side. Zero heap allocation
/// — every glyph is a direct buffer write, clipped to `area`, so
/// degenerate sizes never panic.
pub fn render_observatory(
    f: &mut Frame,
    area: Rect,
    tier_idx: usize,
    planet_name: &str,
    status: PlanetStatus,
    t: Duration,
    reduced_motion: bool,
) {
    let Some(cfg) = PLANET_CONFIGS.get(tier_idx) else {
        return;
    };
    let buf = f.buffer_mut();

    // Body split (design: 62/38) — plain integer math, no Layout pass.
    let canvas_w = (area.width as u32 * 62 / 100) as u16;
    let canvas = Rect::new(area.x, area.y, canvas_w, area.height);
    let hud = Rect::new(
        area.x + canvas_w,
        area.y,
        area.width - canvas_w,
        area.height,
    );

    // Disc rows: clamp(min(canvas.h, rows implied by the horizontal
    // budget), 9, 21). `disc_cols(rows) <= budget` iff
    // `rows <= (budget + 1) / CELL_ASPECT` since
    // `disc_cols(rows) = rows * CELL_ASPECT - 1`; the margin reserves
    // horizontal room for the moon's orbit and the ring tips.
    let width_budget = canvas.width.saturating_sub(HORIZONTAL_MARGIN);
    let rows_from_width = (width_budget + 1) / CELL_ASPECT;
    let fit = canvas.height.min(rows_from_width);
    let rows = fit.clamp(OBS_DISC_MIN, OBS_DISC_MAX);
    let cols = disc_cols(rows);
    let (radius_x, radius_y) = (cols / 2, rows / 2);
    let cx = canvas.x as i32 + canvas.width as i32 / 2;
    let cy = canvas.y as i32 + canvas.height as i32 / 2;

    // 1. Dotted orbit trace in Theme::MUTED — painted first so the
    //    planet, rings, and moon draw over it; dots over the disc's
    //    bounding box are skipped (the trace passes behind).
    let orbit_x = radius_x as f32 + (MOON_ORBIT * CELL_ASPECT as i32) as f32;
    let orbit_y = (radius_y as i32 + MOON_ORBIT) as f32 * MOON_SQUASH;
    let moon_theta = std::f32::consts::TAU * rotation_phase(t, MOON_PERIOD, reduced_motion);
    for i in 0..TRACE_SAMPLES {
        let theta = std::f32::consts::TAU * i as f32 / TRACE_SAMPLES as f32;
        let dx = orbit_x * theta.cos();
        let dy = -orbit_y * theta.sin();
        if dx.abs() as i32 <= radius_x as i32 && dy.abs() as i32 <= radius_y as i32 {
            continue;
        }
        paint(
            buf,
            canvas,
            cx + dx.round() as i32,
            cy + dy.round() as i32,
            '·',
            Theme::MUTED,
        );
    }

    // 2. The shaded disc. Rim glow stays on limb cells only: the
    //    observatory paints rim cells in the day tone even when their
    //    ramp step is deep in the night floor, so the silhouette edge
    //    always glows.
    let mut cells = [ShadedCell::BLANK; OBS_DISC_CELLS];
    let phase = rotation_phase(t, cfg.rotation_period, reduced_motion);
    shade_disc(cfg, status, phase, cols, rows, &mut cells);

    let day = tone(cfg.palette.primary);
    let night = tone(cfg.palette.secondary);
    let top_x = cx - radius_x as i32;
    let top_y = cy - radius_y as i32;
    for row in 0..rows as i32 {
        for col in 0..cols as i32 {
            let cell = cells[row as usize * cols as usize + col as usize];
            let Some(step) = cell.step else {
                continue;
            };
            let color = if cell.rim || step >= DAY_STEP_MIN {
                day
            } else {
                night
            };
            paint(
                buf,
                canvas,
                top_x + col,
                top_y + row,
                RAMP[step as usize],
                color,
            );
        }
    }

    // 3. Ring band on the equator row: inner arc `≡`, Cassini division
    //    gap at `Rx + CASSINI_OFFSET*CELL_ASPECT`, outer arc `─`. The
    //    west arc passes in front of the planet (painted over the disc),
    //    the east arc behind it (the disc cells win).
    let ring_reach = radius_x as i32 + RING_SPAN * CELL_ASPECT as i32;
    let cassini = radius_x as i32 + CASSINI_OFFSET * CELL_ASPECT as i32;
    for dx in -ring_reach..=ring_reach {
        if dx.abs() == cassini {
            continue; // the Cassini division: a blank gap in the band
        }
        if in_disc(dx, 0, cols, rows) && dx >= 0 {
            continue; // east arc passes behind the planet
        }
        let glyph = if dx.abs() <= radius_x as i32 + 1 {
            '≡'
        } else {
            '─'
        };
        paint(buf, canvas, cx + dx, cy, glyph, Theme::ACCENT);
    }

    // 4. The moon: on the far side of the orbit it hides behind the
    //    disc; on the near side it draws in front of disc and trace.
    let moon_x = cx + (orbit_x * moon_theta.cos()).round() as i32;
    let moon_y = cy - (orbit_y * moon_theta.sin()).round() as i32;
    let moon_behind = moon_theta.sin() > 0.0;
    if !(moon_behind && in_disc(moon_x - cx, moon_y - cy, cols, rows)) {
        paint(buf, canvas, moon_x, moon_y, '☾', Theme::TEXT);
    }

    // 5. Telemetry HUD.
    render_hud(buf, hud, planet_name, status, cfg.rotation_period);
}

/// Whether a cell at `dx`/`dy` offsets from the disc center lies inside
/// the painted silhouette: the integer mirror of `sample_sphere`'s unit
/// disc test `(2dx/cols)^2 + (2dy/rows)^2 <= 1`, cross-multiplied to
/// stay in integer math (half-cell off for even `cols`/`rows`).
fn in_disc(dx: i32, dy: i32, cols: u16, rows: u16) -> bool {
    let (c, r) = (cols as i64, rows as i64);
    let (dx, dy) = (dx as i64, dy as i64);
    (2 * dx) * (2 * dx) * r * r + (2 * dy) * (2 * dy) * c * c <= c * c * r * r
}

/// Spanish status badge for the observatory HUD (mirrors the galaxy
/// map's `planet_style` badges in `ui.rs`).
const fn status_badge(status: PlanetStatus) -> &'static str {
    match status {
        PlanetStatus::Conquered => "CONQUISTADO",
        PlanetStatus::Current => "DESTINO ACTUAL",
        PlanetStatus::InProgress => "EN CURSO",
        PlanetStatus::Unexplored => "SIN EXPLORAR",
    }
}

/// Draws the HUD panel: a bordered box titled ` OBSERVATORIO ORBITAL `
/// with one telemetry line per planet fact. Direct buffer writes —
/// zero heap, clipped to `hud`.
fn render_hud(
    buf: &mut Buffer,
    hud: Rect,
    planet_name: &str,
    status: PlanetStatus,
    rotation_period: Duration,
) {
    if hud.width < 2 || hud.height < 2 {
        return; // no room for even the border
    }
    let right = hud.x + hud.width - 1;
    let bottom = hud.y + hud.height - 1;
    for x in hud.x..=right {
        paint(buf, hud, x as i32, hud.y as i32, '─', Theme::PRIMARY);
        paint(buf, hud, x as i32, bottom as i32, '─', Theme::PRIMARY);
    }
    for y in hud.y..=bottom {
        paint(buf, hud, hud.x as i32, y as i32, '│', Theme::PRIMARY);
        paint(buf, hud, right as i32, y as i32, '│', Theme::PRIMARY);
    }
    paint(buf, hud, hud.x as i32, hud.y as i32, '┌', Theme::PRIMARY);
    paint(buf, hud, right as i32, hud.y as i32, '┐', Theme::PRIMARY);
    paint(buf, hud, hud.x as i32, bottom as i32, '└', Theme::PRIMARY);
    paint(buf, hud, right as i32, bottom as i32, '┘', Theme::PRIMARY);
    write_text(
        buf,
        hud,
        hud.x as i32 + 1,
        hud.y as i32,
        " OBSERVATORIO ORBITAL ",
        Theme::PRIMARY,
    );

    let mut row = hud.y as i32 + 2;
    telemetry(buf, hud, row, "PLANETA: ", planet_name);
    row += 1;
    telemetry(buf, hud, row, "ESTADO: ", status_badge(status));
    row += 1;
    let mut x = write_text(buf, hud, hud.x as i32 + 2, row, "ROTACIÓN: ", Theme::MUTED);
    x = write_u64(
        buf,
        hud,
        x,
        row,
        rotation_period.as_millis() as u64,
        Theme::TEXT,
    );
    write_text(buf, hud, x, row, " MS", Theme::TEXT);
    row += 1;
    telemetry(buf, hud, row, "ANILLOS: ", "R+4");
    row += 1;
    telemetry(buf, hud, row, "SATÉLITE: ", "ÓRBITA R+6");
}

/// Writes one telemetry line inside the HUD: muted label + bright
/// value, clipped to `hud`, skipped past the bottom border.
fn telemetry(buf: &mut Buffer, hud: Rect, row: i32, label: &str, value: &str) {
    if row >= hud.y as i32 + hud.height as i32 - 1 {
        return;
    }
    let x = write_text(buf, hud, hud.x as i32 + 2, row, label, Theme::MUTED);
    write_text(buf, hud, x, row, value, Theme::TEXT);
}

/// Writes `text` left to right starting at (`x`, `y`), clipped to
/// `area`; returns the column just past the last written character.
fn write_text(buf: &mut Buffer, area: Rect, x: i32, y: i32, text: &str, color: Color) -> i32 {
    let mut cx = x;
    for ch in text.chars() {
        paint(buf, area, cx, y, ch, color);
        cx += 1;
    }
    cx
}

/// Writes `value` in decimal starting at (`x`, `y`), clipped to
/// `area`; returns the column just past the last digit. Digits are
/// built in a stack buffer — zero heap.
fn write_u64(buf: &mut Buffer, area: Rect, x: i32, y: i32, value: u64, color: Color) -> i32 {
    let mut digits = [b'0'; 20];
    let mut len = 0usize;
    let mut v = value;
    loop {
        digits[19 - len] = b'0' + (v % 10) as u8;
        len += 1;
        v /= 10;
        if v == 0 {
            break;
        }
    }
    let mut cx = x;
    for &digit in &digits[20 - len..] {
        paint(buf, area, cx, y, digit as char, color);
        cx += 1;
    }
    cx
}

/// Buffer write clipped to `area`: the clip guard and the direct write
/// in one panic-free step (negative offsets simply drop).
fn paint(buf: &mut Buffer, area: Rect, x: i32, y: i32, glyph: char, color: Color) {
    if x < area.x as i32 || y < area.y as i32 {
        return;
    }
    let (ux, uy) = (x as u16, y as u16);
    if ux - area.x >= area.width || uy - area.y >= area.height {
        return;
    }
    if let Some(cell) = buf.cell_mut(Position::new(ux, uy)) {
        cell.set_char(glyph).set_fg(color);
    }
}

#[cfg(test)]
mod tests {
    use super::{MOON_PERIOD, render_map_card, render_observatory};
    use crate::core::model::PlanetStatus;
    use crate::tui::planets::{RAMP, TIER_PALETTES, disc_cols};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Position, Rect};
    use ratatui::style::Color;
    use std::time::Duration;

    /// A painted lane cell: buffer position, ramp glyph, fg color.
    type Painted = (u16, u16, char, Color);

    /// Renders one map card into a fresh `TestBackend` buffer.
    fn draw_map_card(
        width: u16,
        height: u16,
        lane: Rect,
        tier_idx: usize,
        status: PlanetStatus,
        t: Duration,
        reduced_motion: bool,
    ) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|f| render_map_card(f.buffer_mut(), lane, tier_idx, status, t, reduced_motion))
            .unwrap();
        terminal.backend().buffer().clone()
    }

    /// Non-blank cells inside `lane`, collected row-major over the whole
    /// buffer (so leaks outside `lane` are visible to other assertions).
    fn lane_painted(buf: &Buffer, lane: Rect) -> Vec<Painted> {
        let mut painted = Vec::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                let cell = buf.cell(Position::new(x, y)).unwrap();
                if cell.symbol() != " " && rect_contains(lane, x, y) {
                    let glyph = cell.symbol().chars().next().unwrap_or(' ');
                    painted.push((x, y, glyph, cell.fg));
                }
            }
        }
        painted
    }

    /// Every buffer cell outside `lane` must still be blank.
    fn assert_outside_lane_blank(buf: &Buffer, lane: Rect) {
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                if rect_contains(lane, x, y) {
                    continue;
                }
                let cell = buf.cell(Position::new(x, y)).unwrap();
                assert_eq!(
                    cell.symbol(),
                    " ",
                    "cell ({x},{y}) outside the lane must stay blank"
                );
            }
        }
    }

    fn rect_contains(area: Rect, x: u16, y: u16) -> bool {
        x >= area.x && y >= area.y && x - area.x < area.width && y - area.y < area.height
    }

    #[test]
    fn map_card_paints_only_ramp_glyphs_inside_the_lane() {
        let lane = Rect::new(2, 1, 12, 7);
        let buf = draw_map_card(
            40,
            12,
            lane,
            0,
            PlanetStatus::Conquered,
            Duration::ZERO,
            false,
        );

        let painted = lane_painted(&buf, lane);
        assert!(
            !painted.is_empty(),
            "the disc must paint at least one lane cell"
        );
        for &(x, y, glyph, _) in &painted {
            assert!(
                RAMP.contains(&glyph),
                "glyph {glyph:?} at ({x},{y}) must be a ramp step"
            );
        }
        assert_outside_lane_blank(&buf, lane);
    }

    #[test]
    fn map_card_clipped_at_buffer_edge_does_not_panic_or_leak() {
        // The lane overflows the 40x12 buffer on both the right and the
        // bottom edge: only a 2-column, 4-row slice of the disc is visible.
        let lane = Rect::new(36, 8, 12, 7);
        let buf = draw_map_card(
            40,
            12,
            lane,
            0,
            PlanetStatus::Conquered,
            Duration::ZERO,
            false,
        );

        let painted = lane_painted(&buf, lane);
        assert!(
            !painted.is_empty(),
            "the visible slice of a clipped card must still paint"
        );
        for &(x, y, glyph, _) in &painted {
            assert!(
                RAMP.contains(&glyph),
                "glyph {glyph:?} at ({x},{y}) must be a ramp step"
            );
        }
        assert_outside_lane_blank(&buf, lane);
    }

    #[test]
    fn map_card_degenerate_one_by_one_lane_paints_without_panicking() {
        let lane = Rect::new(3, 2, 1, 1);
        let buf = draw_map_card(
            8,
            5,
            lane,
            6,
            PlanetStatus::Current,
            Duration::from_millis(123),
            false,
        );

        let painted = lane_painted(&buf, lane);
        assert_eq!(painted.len(), 1, "the 1x1 disc is its own center cell");
        let &(_, _, glyph, color) = &painted[0];
        assert!(
            RAMP.contains(&glyph),
            "1x1 disc center paints a ramp glyph, got {glyph:?}"
        );
        assert!(
            matches!(color, Color::Rgb(..)),
            "truecolor-only fg, got {color:?}"
        );
    }

    #[test]
    fn map_card_rotation_advances_between_ticks() {
        let lane = Rect::new(0, 0, 12, 7);
        let tier = 2; // ORTOGRAFÍA — cratered, 5500 ms period
        let first = draw_map_card(
            20,
            9,
            lane,
            tier,
            PlanetStatus::InProgress,
            Duration::ZERO,
            false,
        );
        let second = draw_map_card(
            20,
            9,
            lane,
            tier,
            PlanetStatus::InProgress,
            Duration::from_millis(2750),
            false,
        );

        let a = lane_painted(&first, lane);
        let b = lane_painted(&second, lane);
        assert!(!a.is_empty() && !b.is_empty(), "both frames must draw");
        assert_ne!(
            a, b,
            "half a rotation (t=0 vs t=period/2) must move at least one lane cell"
        );
    }

    #[test]
    fn map_card_reduced_motion_freezes_the_sphere() {
        let lane = Rect::new(0, 0, 12, 7);
        let tier = 2; // 5500 ms period — would spin freely without D10
        let first = draw_map_card(
            20,
            9,
            lane,
            tier,
            PlanetStatus::InProgress,
            Duration::ZERO,
            true,
        );
        let second = draw_map_card(
            20,
            9,
            lane,
            tier,
            PlanetStatus::InProgress,
            Duration::from_millis(8421),
            true,
        );

        let a = lane_painted(&first, lane);
        assert!(!a.is_empty(), "D10 still renders a static sphere");
        assert_eq!(
            a,
            lane_painted(&second, lane),
            "reduced motion must pin the sphere at the t=0 phase"
        );
    }

    #[test]
    fn map_card_tiers_paint_pairwise_distinct_rgb_sets() {
        let lane = Rect::new(0, 0, 12, 7);
        let mut sets: Vec<Vec<Color>> = Vec::new();
        for tier in 0..7 {
            let buf = draw_map_card(
                20,
                9,
                lane,
                tier,
                PlanetStatus::Conquered,
                Duration::ZERO,
                true,
            );
            let painted = lane_painted(&buf, lane);
            assert!(!painted.is_empty(), "tier {tier} must paint its lane");

            // Exactly the palette's two truecolor tones: day and night.
            let palette = TIER_PALETTES[tier];
            let day = Color::Rgb(palette.primary.r, palette.primary.g, palette.primary.b);
            let night = Color::Rgb(
                palette.secondary.r,
                palette.secondary.g,
                palette.secondary.b,
            );
            let mut colors: Vec<Color> = Vec::new();
            for &(_, _, _, color) in &painted {
                assert!(
                    matches!(color, Color::Rgb(..)),
                    "tier {tier} painted non-truecolor fg {color:?}"
                );
                if !colors.contains(&color) {
                    colors.push(color);
                }
            }
            assert_eq!(colors.len(), 2, "tier {tier} must use both palette tones");
            assert!(
                colors.contains(&day) && colors.contains(&night),
                "tier {tier} painted {colors:?}, expected day {day:?} and night {night:?}"
            );
            sets.push(colors);
        }

        for (i, left) in sets.iter().enumerate() {
            for (j, right) in sets.iter().enumerate().skip(i + 1) {
                assert_ne!(
                    left,
                    right,
                    "tiers {} and {} paint the same fg set",
                    i + 1,
                    j + 1
                );
            }
        }
    }

    #[test]
    fn map_card_conquered_day_glyphs_outshine_night() {
        let lane = Rect::new(0, 0, 12, 7);
        let tier = 0;
        let buf = draw_map_card(
            20,
            9,
            lane,
            tier,
            PlanetStatus::Conquered,
            Duration::ZERO,
            true,
        );

        let palette = TIER_PALETTES[tier];
        let day = Color::Rgb(palette.primary.r, palette.primary.g, palette.primary.b);
        let night = Color::Rgb(
            palette.secondary.r,
            palette.secondary.g,
            palette.secondary.b,
        );
        let mut day_steps: Vec<usize> = Vec::new();
        let mut night_steps: Vec<usize> = Vec::new();
        for &(_, _, glyph, color) in &lane_painted(&buf, lane) {
            let step = RAMP
                .iter()
                .position(|ramp| *ramp == glyph)
                .expect("painted glyphs are ramp steps");
            if color == day {
                day_steps.push(step);
            } else if color == night {
                night_steps.push(step);
            } else {
                panic!("unexpected fg {color:?} on a Conquered card");
            }
        }
        assert!(
            !day_steps.is_empty() && !night_steps.is_empty(),
            "Conquered shows both a lit and a shadowed hemisphere"
        );
        assert!(
            night_steps.iter().max().unwrap() < day_steps.iter().min().unwrap(),
            "every day glyph must outshine every night glyph (day {day_steps:?} vs night {night_steps:?})"
        );
    }

    #[test]
    fn map_card_status_transition_relights_the_sphere() {
        let lane = Rect::new(0, 0, 12, 7);
        let tier = 0;
        let before = draw_map_card(
            20,
            9,
            lane,
            tier,
            PlanetStatus::InProgress,
            Duration::ZERO,
            true,
        );
        let after = draw_map_card(
            20,
            9,
            lane,
            tier,
            PlanetStatus::Conquered,
            Duration::ZERO,
            true,
        );

        let a = lane_painted(&before, lane);
        let b = lane_painted(&after, lane);
        assert!(!a.is_empty() && !b.is_empty(), "both statuses must draw");
        assert_ne!(
            a, b,
            "passing the tier's last lesson must relight its card (InProgress -> Conquered)"
        );

        let brightness = |painted: &Vec<Painted>| -> usize {
            painted
                .iter()
                .map(|&(_, _, glyph, _)| RAMP.iter().position(|r| *r == glyph).unwrap())
                .sum()
        };
        assert!(
            brightness(&b) > brightness(&a),
            "Conquered lighting must be brighter than InProgress"
        );
    }

    // -----------------------------------------------------------------
    // Observatory (docked-planet view)
    // -----------------------------------------------------------------

    /// The observatory's left canvas / right HUD rects for a full-frame
    /// `width x height` render — pins the design's 62/38 body split.
    fn observatory_regions(width: u16, height: u16) -> (Rect, Rect) {
        let canvas_w = width * 62 / 100;
        let canvas = Rect::new(0, 0, canvas_w, height);
        let hud = Rect::new(canvas_w, 0, width - canvas_w, height);
        (canvas, hud)
    }

    /// Renders the docked-planet observatory into a fresh `TestBackend`
    /// buffer covering the full `width x height` frame.
    fn draw_observatory(
        width: u16,
        height: u16,
        tier_idx: usize,
        name: &str,
        status: PlanetStatus,
        t: Duration,
        reduced_motion: bool,
    ) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|f| render_observatory(f, f.area(), tier_idx, name, status, t, reduced_motion))
            .unwrap();
        terminal.backend().buffer().clone()
    }

    /// Concatenates the symbols of the cells inside `rect`.
    fn rect_symbols(buf: &Buffer, rect: Rect) -> String {
        let mut out = String::new();
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                if let Some(cell) = buf.cell(Position::new(x, y)) {
                    out.push_str(cell.symbol());
                }
            }
        }
        out
    }

    /// Buffer positions, row-major inside `rect`, whose cell paints `glyph`.
    fn glyph_positions(buf: &Buffer, rect: Rect, glyph: char) -> Vec<(u16, u16)> {
        let mut found = Vec::new();
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                if let Some(cell) = buf.cell(Position::new(x, y)) {
                    if cell.symbol().chars().next() == Some(glyph) {
                        found.push((x, y));
                    }
                }
            }
        }
        found
    }

    /// Inclusive bounding box `(min_x, min_y, max_x, max_y)` of the disc:
    /// the canvas cells painted in the tier palette's two truecolor tones
    /// (every other observatory element uses a Theme chrome color).
    fn disc_bbox(buf: &Buffer, canvas: Rect, tier: usize) -> (u16, u16, u16, u16) {
        let palette = TIER_PALETTES[tier];
        let day = Color::Rgb(palette.primary.r, palette.primary.g, palette.primary.b);
        let night = Color::Rgb(
            palette.secondary.r,
            palette.secondary.g,
            palette.secondary.b,
        );
        let mut bbox: Option<(u16, u16, u16, u16)> = None;
        for y in canvas.y..canvas.y + canvas.height {
            for x in canvas.x..canvas.x + canvas.width {
                let Some(cell) = buf.cell(Position::new(x, y)) else {
                    continue;
                };
                if cell.fg == day || cell.fg == night {
                    bbox = Some(match bbox {
                        Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
                        None => (x, y, x, y),
                    });
                }
            }
        }
        bbox.expect("the observatory disc must paint palette-colored cells")
    }

    #[test]
    fn observatory_dock_frame_shows_rings_moon_and_hud() {
        let (canvas, hud) = observatory_regions(100, 24);
        let buf = draw_observatory(
            100,
            24,
            0,
            "CIMIENTOS",
            PlanetStatus::Conquered,
            Duration::ZERO,
            false,
        );

        let canvas_symbols = rect_symbols(&buf, canvas);
        assert!(
            canvas_symbols.contains('≡'),
            "the inner ring band must paint ≡ glyphs: {canvas_symbols}"
        );
        assert!(
            canvas_symbols.contains('─'),
            "the outer ring band must paint ─ glyphs: {canvas_symbols}"
        );
        assert!(
            !glyph_positions(&buf, canvas, '☾').is_empty(),
            "the moon must paint at least one cell"
        );

        // The disc itself paints palette-colored ramp cells within the
        // design ceiling: at most `disc_cols(OBS_DISC_MAX)` columns wide
        // and `OBS_DISC_MAX` rows tall.
        let (x0, y0, x1, y1) = disc_bbox(&buf, canvas, 0);
        assert!(
            x1 - x0 < disc_cols(21) && y1 - y0 < 21,
            "disc bbox ({x0},{y0})..({x1},{y1}) must respect the design ceiling"
        );

        let hud_symbols = rect_symbols(&buf, hud);
        for label in ["OBSERVATORIO", "PLANETA", "CIMIENTOS", "CONQUISTADO"] {
            assert!(hud_symbols.contains(label), "HUD must show {label}");
        }
    }

    #[test]
    fn observatory_ring_band_shows_a_cassini_division_gap() {
        let (canvas, _) = observatory_regions(100, 24);
        let buf = draw_observatory(
            100,
            24,
            0,
            "CIMIENTOS",
            PlanetStatus::Conquered,
            Duration::ZERO,
            false,
        );

        // The ring band is the only row painting ≡ glyphs.
        let ring_cells = glyph_positions(&buf, canvas, '≡');
        assert!(
            ring_cells.len() >= 2,
            "the ring band must paint multiple ≡ cells"
        );
        let ring_y = ring_cells[0].1;
        assert!(
            ring_cells.iter().all(|&(_, y)| y == ring_y),
            "ring glyphs must share one equator row"
        );

        let is_ring = |x: u16| {
            buf.cell(Position::new(x, ring_y))
                .is_some_and(|c| c.symbol() == "≡" || c.symbol() == "─")
        };
        let row_has_outer = (canvas.x..canvas.x + canvas.width).any(|x| {
            buf.cell(Position::new(x, ring_y))
                .is_some_and(|c| c.symbol() == "─")
        });
        assert!(row_has_outer, "the ring row must also paint ─ glyphs");

        // Cassini division: at least one blank cell on the ring row is
        // flanked on both sides by ring glyphs, splitting the band.
        let mut divisions = 0;
        for x in (canvas.x + 1)..(canvas.x + canvas.width - 1) {
            let blank = buf
                .cell(Position::new(x, ring_y))
                .is_some_and(|c| c.symbol() == " ");
            if blank && is_ring(x - 1) && is_ring(x + 1) {
                divisions += 1;
            }
        }
        assert!(
            divisions >= 1,
            "the ring band must be split by a >=1-cell Cassini gap"
        );
    }

    #[test]
    fn observatory_moon_orbits_outside_the_disc_and_moves() {
        let (canvas, _) = observatory_regions(100, 24);
        let east = draw_observatory(
            100,
            24,
            0,
            "CIMIENTOS",
            PlanetStatus::Conquered,
            Duration::ZERO,
            false,
        );
        let west = draw_observatory(
            100,
            24,
            0,
            "CIMIENTOS",
            PlanetStatus::Conquered,
            MOON_PERIOD / 2,
            false,
        );

        let moon_east = glyph_positions(&east, canvas, '☾');
        let moon_west = glyph_positions(&west, canvas, '☾');
        assert_eq!(moon_east.len(), 1, "exactly one moon cell at t=0");
        assert_eq!(moon_west.len(), 1, "exactly one moon cell at half an orbit");
        assert_ne!(
            moon_east[0], moon_west[0],
            "the moon must move across ticks"
        );

        // Both sampled positions stay outside the disc silhouette.
        for (buf, moon) in [(&east, moon_east[0]), (&west, moon_west[0])] {
            let (x0, y0, x1, y1) = disc_bbox(buf, canvas, 0);
            let (mx, my) = moon;
            assert!(
                mx < x0 || mx > x1 || my < y0 || my > y1,
                "moon at ({mx},{my}) must stay outside the disc bbox ({x0},{y0})..({x1},{y1})"
            );
        }
    }

    #[test]
    fn observatory_moon_hides_behind_and_crosses_in_front_of_the_disc() {
        let (canvas, _) = observatory_regions(100, 24);
        // A quarter orbit puts the moon on the far side above the planet
        // (inside the silhouette: hidden); three quarters puts it on the
        // near side below it (drawn in front of the disc).
        let far = draw_observatory(
            100,
            24,
            0,
            "CIMIENTOS",
            PlanetStatus::Conquered,
            MOON_PERIOD / 4,
            false,
        );
        let near = draw_observatory(
            100,
            24,
            0,
            "CIMIENTOS",
            PlanetStatus::Conquered,
            MOON_PERIOD * 3 / 4,
            false,
        );

        assert!(
            glyph_positions(&far, canvas, '☾').is_empty(),
            "the far-side moon must be occluded by the planet"
        );

        let front = glyph_positions(&near, canvas, '☾');
        assert_eq!(front.len(), 1, "the near-side moon must draw in front");
        let (x0, y0, x1, y1) = disc_bbox(&near, canvas, 0);
        let (mx, my) = front[0];
        assert!(
            mx >= x0 && mx <= x1 && my >= y0 && my <= y1,
            "the near-side moon at ({mx},{my}) must cross in front of the disc bbox ({x0},{y0})..({x1},{y1})"
        );
    }

    #[test]
    fn observatory_reduced_motion_freezes_a_fully_drawn_frame() {
        let (canvas, hud) = observatory_regions(100, 24);
        let first = draw_observatory(
            100,
            24,
            0,
            "CIMIENTOS",
            PlanetStatus::Conquered,
            Duration::ZERO,
            true,
        );
        let second = draw_observatory(
            100,
            24,
            0,
            "CIMIENTOS",
            PlanetStatus::Conquered,
            Duration::from_millis(13_457),
            true,
        );

        assert_eq!(first, second, "D10 must pin the observatory frame");

        // Identical AND fully drawn: rings, moon, disc, and HUD present.
        let canvas_symbols = rect_symbols(&first, canvas);
        assert!(
            canvas_symbols.contains('≡') && canvas_symbols.contains('─'),
            "the frozen frame still paints its rings"
        );
        assert!(
            !glyph_positions(&first, canvas, '☾').is_empty(),
            "the frozen frame still paints its moon"
        );
        assert!(
            rect_symbols(&first, hud).contains("OBSERVATORIO"),
            "the frozen frame still paints the HUD"
        );
        let _ = disc_bbox(&first, canvas, 0); // panics without disc cells
    }

    #[test]
    fn observatory_smallest_area_clips_and_stays_inside() {
        // A 20x6 observatory area floating inside a larger buffer: the
        // disc, rings, and HUD clip to the area and nothing leaks out.
        let mut terminal = Terminal::new(TestBackend::new(34, 14)).unwrap();
        let area = Rect::new(5, 3, 20, 6);
        terminal
            .draw(|f| {
                render_observatory(
                    f,
                    area,
                    0,
                    "CIMIENTOS",
                    PlanetStatus::Conquered,
                    Duration::ZERO,
                    false,
                )
            })
            .unwrap();
        let buf = terminal.backend().buffer().clone();

        let mut painted_in_area = 0;
        for y in 0..14 {
            for x in 0..34 {
                let symbol = buf.cell(Position::new(x, y)).unwrap().symbol();
                if symbol != " " {
                    assert!(
                        rect_contains(area, x, y),
                        "glyph {symbol:?} at ({x},{y}) leaked outside the observatory area"
                    );
                    painted_in_area += 1;
                }
            }
        }
        assert!(
            painted_in_area > 0,
            "the clipped frame must still paint inside its area"
        );

        // A degenerate 1x1 area also renders panic-free.
        let mut degenerate = Terminal::new(TestBackend::new(2, 2)).unwrap();
        degenerate
            .draw(|f| {
                render_observatory(
                    f,
                    Rect::new(0, 0, 1, 1),
                    0,
                    "X",
                    PlanetStatus::Unexplored,
                    Duration::ZERO,
                    false,
                )
            })
            .unwrap();
    }

    #[test]
    fn observatory_disc_diameter_follows_the_clamp_formula() {
        // rows = clamp(min(canvas.h, rows implied by the horizontal
        // budget), 9, 21); cols = disc_cols(rows). A large canvas hits
        // the row ceiling, a height-limited canvas paints canvas.h rows,
        // a width-limited canvas paints the rows the horizontal budget
        // allows, and a tiny canvas floors at 9 rows (drawing larger
        // than the strict fit, clipped by the area guard). Every case's
        // column span must equal `disc_cols` of its row span.
        let cases: [(u16, u16, u16); 4] = [
            (110, 30, 21), // rows_from_width 22 vs height 30 -> ceiling 21
            (100, 16, 16), // height 16 limits below the ceiling
            (76, 30, 12),  // canvas width 47 -> rows_from_width 12
            (30, 10, 9),   // canvas width 18 -> rows_from_width 0, floors at 9
        ];
        for (width, height, expected_rows) in cases {
            let (canvas, _) = observatory_regions(width, height);
            let buf = draw_observatory(
                width,
                height,
                0,
                "CIMIENTOS",
                PlanetStatus::Conquered,
                Duration::ZERO,
                true,
            );
            let (x0, y0, x1, y1) = disc_bbox(&buf, canvas, 0);
            let expected_cols = disc_cols(expected_rows);
            assert_eq!(
                y1 - y0 + 1,
                expected_rows,
                "{width}x{height}: disc must span {expected_rows} rows (got {})",
                y1 - y0 + 1
            );
            assert_eq!(
                x1 - x0 + 1,
                expected_cols,
                "{width}x{height}: disc must span {expected_cols} columns (got {})",
                x1 - x0 + 1
            );
        }
    }

    /// The map card's ray-cast disc must silhouette wider than it is
    /// tall on screen: the widest painted row spans more columns than
    /// the tallest painted column spans rows.
    #[test]
    fn map_card_disc_is_wider_than_tall() {
        let lane = Rect::new(0, 0, 14, 7);
        let buf = draw_map_card(
            20,
            9,
            lane,
            0,
            PlanetStatus::Conquered,
            Duration::ZERO,
            true,
        );
        let painted = lane_painted(&buf, lane);
        assert!(!painted.is_empty(), "the disc must paint at least one cell");

        let widest_row = painted
            .iter()
            .fold(std::collections::HashMap::new(), |mut rows, &(x, y, ..)| {
                let (min_x, max_x) = rows.entry(y).or_insert((x, x));
                *min_x = (*min_x).min(x);
                *max_x = (*max_x).max(x);
                rows
            })
            .values()
            .map(|(min_x, max_x)| max_x - min_x + 1)
            .max()
            .expect("at least one painted row");
        let tallest_col = painted
            .iter()
            .fold(std::collections::HashMap::new(), |mut cols, &(x, y, ..)| {
                let (min_y, max_y) = cols.entry(x).or_insert((y, y));
                *min_y = (*min_y).min(y);
                *max_y = (*max_y).max(y);
                cols
            })
            .values()
            .map(|(min_y, max_y)| max_y - min_y + 1)
            .max()
            .expect("at least one painted column");

        assert!(
            widest_row > tallest_col,
            "the map card disc must paint wider than tall: widest row {widest_row} \
             vs tallest column {tallest_col}"
        );
    }

    /// The observatory disc must likewise silhouette wider than tall, and
    /// the ring band's tips must still land inside the canvas rather
    /// than being clipped away by too tight a horizontal budget.
    #[test]
    fn observatory_disc_is_wider_than_tall_and_ring_tips_fit_the_canvas() {
        let (canvas, _) = observatory_regions(100, 24);
        let buf = draw_observatory(
            100,
            24,
            0,
            "CIMIENTOS",
            PlanetStatus::Conquered,
            Duration::ZERO,
            true,
        );

        let (x0, y0, x1, y1) = disc_bbox(&buf, canvas, 0);
        let (cols_span, rows_span) = (x1 - x0 + 1, y1 - y0 + 1);
        assert!(
            cols_span > rows_span,
            "the observatory disc must paint wider than tall: {cols_span} cols vs {rows_span} rows"
        );

        // Both ring glyphs (`≡` and `─`) must land strictly inside the
        // canvas: none clipped away at the left/right edge.
        for glyph in ['≡', '─'] {
            let positions = glyph_positions(&buf, canvas, glyph);
            assert!(
                !positions.is_empty(),
                "ring glyph {glyph:?} must paint at least one cell"
            );
            for (x, _) in positions {
                assert!(
                    x > canvas.x && x < canvas.x + canvas.width - 1,
                    "ring glyph {glyph:?} at column {x} must stay inside the canvas \
                     ({}..{})",
                    canvas.x,
                    canvas.x + canvas.width
                );
            }
        }
    }
}
