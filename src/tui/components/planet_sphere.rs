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
use crate::tui::planets::{PLANET_CONFIGS, RAMP, Rgb, ShadedCell, rotation_phase, shade_disc};

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;

/// Map-card disc diameter: lanes are 7 rows tall, so the disc is 7x7 —
/// the design's map-card sphere (the observatory renders larger discs).
const MAP_CARD_DISC_DIAMETER: u16 = 7;

/// Stack backing for one map-card disc (diameter-squared cells) — the
/// render path allocates nothing on the heap.
const MAP_CARD_CELLS: usize = MAP_CARD_DISC_DIAMETER as usize * MAP_CARD_DISC_DIAMETER as usize;

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
    let diameter = lane.width.min(lane.height).min(MAP_CARD_DISC_DIAMETER);
    let mut cells = [ShadedCell::BLANK; MAP_CARD_CELLS];
    let phase = rotation_phase(t, cfg.rotation_period, reduced_motion);
    shade_disc(cfg, status, phase, diameter, &mut cells);

    // The palette -> terminal-color bridge (`tone` below) is the only
    // place pure `planets::palette` values become ratatui colors.
    let palette = cfg.palette;
    let day = tone(palette.primary);
    let night = tone(palette.secondary);

    let offset_x = (lane.width - diameter) / 2;
    let offset_y = (lane.height - diameter) / 2;
    for row in 0..diameter {
        for col in 0..diameter {
            let Some(step) = cells[row as usize * diameter as usize + col as usize].step else {
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

#[cfg(test)]
mod tests {
    use super::render_map_card;
    use crate::core::model::PlanetStatus;
    use crate::tui::planets::{RAMP, TIER_PALETTES};
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
}
