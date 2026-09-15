//! Pure layout geometry for the tier-map "galaxy" planet view.
//!
//! This module contains no rendering logic (no `Frame`/`Widget`) and no
//! dependency on [`crate::tui::app::App`]; it only computes [`Rect`]s from
//! an input area so the layout math can be unit tested in isolation.

use crate::core::model::{Lesson, Section};
use crate::tui::animation::ShipPhase;
use ratatui::layout::{Position, Rect};

/// Minimum map-body width (in columns) required to render the full galaxy
/// map. Lowered from 64 so common 80-column terminals reach Full mode
/// (80 * 62% map-body split ≈ 50 >= 46), where the fixed-step band shows
/// ~3 planet cards.
pub const MIN_FULL_WIDTH: u16 = 46;
/// Height (in rows) of the ship sprite ([`crate::tui::ascii::AsciiArt::SHIP_FRAMES`]
/// is 3 rows tall); the horizontal band reserves exactly this many top rows
/// as the ship's flight lane.
pub const SHIP_GUTTER_HEIGHT: u16 = 3;
/// Height (in rows) of the planet sprite lane: the top rows of each
/// Full-mode card, sized for the 7-row ray-cast planet disc (diameter 7,
/// the odd height with a center row the sphere math needs).
pub const SPRITE_LANE_HEIGHT: u16 = 7;
/// Number of info lines stacked below the sprite lane in each Full-mode
/// card (D8 anatomy): marker+name / glyph+badge / progress·meta.
pub const INFO_LINE_COUNT: u16 = 3;
/// Height (in rows) of a single planet card in [`MapMode::Full`]: the
/// sprite lane on top plus the info lines below it.
pub const PLANET_CARD_HEIGHT: u16 = SPRITE_LANE_HEIGHT + INFO_LINE_COUNT;
/// Minimum map-body height (in rows) required to render the full galaxy
/// map: one ship-gutter row block plus one card row block.
pub const MIN_FULL_HEIGHT: u16 = SHIP_GUTTER_HEIGHT + PLANET_CARD_HEIGHT;
/// Width (in columns) of a single planet card in [`MapMode::Full`].
/// Wide enough for the 12-column sprite lane plus a sliver of info.
pub const PLANET_CARD_WIDTH: u16 = 14;
/// Horizontal gap (in columns) between neighbor planet cards in
/// [`MapMode::Full`]. With the card width this fixes the band step at 16
/// columns: ~3 visible cards on an 80×24 terminal, 4 on 120×40.
pub const PLANET_GAP: u16 = 2;
/// Number of tiers/planets rendered on the galaxy map.
pub const PLANET_COUNT: usize = 7;

/// Rendering density for the galaxy map, selected from the available area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapMode {
    /// Roomy layout: fixed-step planet cards left-to-right below a top
    /// ship-gutter row, panned by a camera-follow window.
    Full,
    /// Cramped layout: seven single-row columns, no gutter.
    Compact,
}

/// Selects [`MapMode::Full`] only when `area` is at least
/// [`MIN_FULL_WIDTH`] x [`MIN_FULL_HEIGHT`]; otherwise [`MapMode::Compact`].
pub fn map_mode(area: Rect) -> MapMode {
    if area.width >= MIN_FULL_WIDTH && area.height >= MIN_FULL_HEIGHT {
        MapMode::Full
    } else {
        MapMode::Compact
    }
}

/// How many fixed-step planet cards fully fit in `width` columns
/// (never more than [`PLANET_COUNT`]).
fn band_capacity(width: u16) -> usize {
    if width < PLANET_CARD_WIDTH {
        return 0;
    }
    let step = PLANET_CARD_WIDTH + PLANET_GAP;
    1 + (width - PLANET_CARD_WIDTH) as usize / step as usize
}

/// First visible planet index so `selected` stays inside a window of
/// `capacity` planets. `capacity == 0` always returns `0` (nothing is
/// visible). Identical algorithm to the deleted `viewport_start`.
pub fn band_viewport_start(selected: usize, capacity: usize) -> usize {
    if capacity == 0 {
        return 0;
    }
    if selected >= capacity {
        selected + 1 - capacity
    } else {
        0
    }
}

/// Always returns exactly [`PLANET_COUNT`] rects, ordered Tier1..Tier7
/// left-to-right. Full mode lays fixed-step cards
/// ([`PLANET_CARD_WIDTH`] + [`PLANET_GAP`]) below the top ship-gutter row,
/// panned so `selected` stays visible; planets outside the window are zero
/// rects. Compact mode lays 7 equal one-row columns with no panning. Never
/// panics, even on degenerate (tiny or zero) areas.
pub fn planet_layout(area: Rect, selected: usize) -> Vec<Rect> {
    match map_mode(area) {
        MapMode::Full => {
            let capacity = band_capacity(area.width).min(PLANET_COUNT);
            let start = band_viewport_start(selected, capacity);
            let cards_y = area.y.saturating_add(SHIP_GUTTER_HEIGHT);
            let step = PLANET_CARD_WIDTH + PLANET_GAP;
            (0..PLANET_COUNT)
                .map(|i| {
                    let slot = i as i64 - start as i64;
                    if capacity == 0 || slot < 0 || slot >= capacity as i64 {
                        Rect::new(area.x, cards_y, 0, 0)
                    } else {
                        Rect::new(
                            area.x + slot as u16 * step,
                            cards_y,
                            PLANET_CARD_WIDTH,
                            PLANET_CARD_HEIGHT,
                        )
                    }
                })
                .collect()
        }
        MapMode::Compact => {
            let w = area.width / PLANET_COUNT as u16;
            (0..PLANET_COUNT)
                .map(|i| Rect::new(area.x + i as u16 * w, area.y, w, 1))
                .collect()
        }
    }
}

/// The ship gutter row in [`MapMode::Full`]: the top [`SHIP_GUTTER_HEIGHT`]
/// rows of `area`, the ship's horizontal flight lane. A zero-height
/// [`Rect`] in [`MapMode::Compact`].
pub fn ship_gutter(area: Rect) -> Rect {
    match map_mode(area) {
        MapMode::Full => Rect::new(
            area.x,
            area.y,
            area.width,
            SHIP_GUTTER_HEIGHT.min(area.height),
        ),
        MapMode::Compact => Rect::new(area.x, area.y, area.width, 0),
    }
}

/// Hit-tests `pos` against the current planet layout for `area` (windowed
/// around `selected`). Returns the index of the card containing `pos`, or
/// `None` for the gutter row or any point outside all cards.
pub fn planet_at(area: Rect, pos: Position, selected: usize) -> Option<usize> {
    planet_layout(area, selected)
        .iter()
        .position(|card| card.width > 0 && card.height > 0 && card.contains(pos))
}

/// Vertical center of the ship sprite for the horizontal band. While
/// [`ShipPhase::Idle`]/[`ShipPhase::Traveling`] the ship parks at the
/// gutter's sprite-lane middle row (`core`); the dock is the vertical move
/// onto the planet: [`ShipPhase::Descending`] lerps core → the target
/// card's top row (`entry` = `card.y`) by `dock_depth`, and
/// [`ShipPhase::Ascending`] reverses it. Never panics: an empty or
/// fully-off-window `cards` slice just leaves the ship parked at `core`.
pub fn ship_y(
    gutter: Rect,
    cards: &[Rect],
    target: usize,
    phase: ShipPhase,
    dock_depth: f32,
) -> f32 {
    let core = gutter.y as f32 + gutter.height as f32 / 2.0;
    let entry = cards
        .get(target.min(cards.len().saturating_sub(1)))
        .filter(|card| card.width > 0 && card.height > 0)
        .map(|card| card.y as f32)
        .unwrap_or(core);
    match phase {
        ShipPhase::Idle | ShipPhase::Traveling => core,
        ShipPhase::Descending => core + (entry - core) * dock_depth,
        ShipPhase::Ascending => entry + (core - entry) * dock_depth,
    }
}

/// Horizontal center of the ship sprite: lerps the visible cards' center
/// columns by the fractional planet index `pos` (the transpose of the old
/// vertical `ship_rect`). Positions outside the visible window clamp to
/// the window's edge cards (off-window cards are zero rects); an empty
/// `cards` slice falls back to `0.0` so callers stay finite. Never panics.
pub fn ship_x(cards: &[Rect], pos: f32) -> f32 {
    let is_visible = |card: &Rect| card.width > 0 && card.height > 0;
    // The visible window is contiguous, so its first/last members bound
    // every flight without collecting an index list (no per-frame alloc).
    let Some(lo) = cards.iter().position(is_visible) else {
        return 0.0;
    };
    let Some(hi) = cards.iter().rposition(is_visible) else {
        return 0.0;
    };
    let center_x = |k: usize| cards[k].x as f32 + cards[k].width as f32 / 2.0;
    let clamped = pos.clamp(lo as f32, hi as f32);
    let i = clamped.floor() as usize;
    let t = clamped.fract();
    let next = (i + 1).min(hi);
    let x0 = center_x(i);
    let x1 = center_x(next);
    x0 + (x1 - x0) * t
}

/// Width (in columns) of the planet sprite lane inside each Full-mode card.
/// Anchors the planet sprite's left edge inside the card; the ship docks
/// down onto this lane from the top gutter row.
pub const SPRITE_LANE_WIDTH: u16 = 12;

/// Top sprite lane of a Full-mode planet card (D8 stacked anatomy): the
/// first [`SPRITE_LANE_HEIGHT`] rows, [`SPRITE_LANE_WIDTH`] columns wide
/// and anchored at the card's left edge. The ship docks down onto this
/// lane from the top gutter row. `sprite_lane ∪ info_column` tiles the
/// card exactly — no overlap, no gap — via inline Rect math (deliberately
/// not nested `Layout`, which can leave rounding gaps).
pub fn sprite_lane(card: Rect) -> Rect {
    let w = SPRITE_LANE_WIDTH.min(card.width);
    let h = SPRITE_LANE_HEIGHT.min(card.height);
    Rect::new(card.x, card.y, w, h)
}

/// Info block below the sprite lane of a Full-mode planet card (D8
/// stacked anatomy): every row under [`sprite_lane`], spanning the full
/// card width. Degrades to a zero-height slice on a card shorter than
/// the lane — still tiling the card without overlap or gap.
pub fn info_column(card: Rect) -> Rect {
    let lane_h = SPRITE_LANE_HEIGHT.min(card.height);
    Rect::new(
        card.x,
        card.y.saturating_add(lane_h),
        card.width,
        card.height.saturating_sub(lane_h),
    )
}

/// One row of the planet lesson list: either a non-selectable section
/// header or a lesson row referencing its FLAT index into
/// [`crate::core::curriculum::Curriculum::all_lessons`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuRow {
    Header(String),
    Lesson(usize),
}

/// Builds display rows for a single tier's lesson list: one [`MenuRow::Header`]
/// before the first lesson of each section, followed by [`MenuRow::Lesson`]
/// rows carrying the FLAT index from `tier_lessons`, preserving input order.
pub fn build_rows(tier_lessons: &[(usize, &Lesson)], sections: &[Section]) -> Vec<MenuRow> {
    let mut rows = Vec::with_capacity(tier_lessons.len() + sections.len());
    let mut current_section: Option<&str> = None;

    for (flat_idx, lesson) in tier_lessons {
        if current_section != Some(lesson.section_id.as_str()) {
            let title = sections
                .iter()
                .find(|s| s.id == lesson.section_id)
                .map(|s| s.title.clone())
                .unwrap_or_else(|| lesson.section_id.clone());
            rows.push(MenuRow::Header(title));
            current_section = Some(lesson.section_id.as_str());
        }
        rows.push(MenuRow::Lesson(*flat_idx));
    }

    rows
}

/// Maps a terminal `row` inside `list_area` (a bordered widget) to the
/// display index it represents, then resolves it against `rows` starting at
/// display index `start`. Returns `Some(flat_idx)` only for a
/// [`MenuRow::Lesson`] row; `None` for headers, borders, or out-of-range rows.
pub fn lesson_row_at(list_area: Rect, start: usize, rows: &[MenuRow], row: u16) -> Option<usize> {
    let inner_top = list_area.y.checked_add(1)?;
    let border_bottom = list_area.y.checked_add(list_area.height)?.checked_sub(1)?;
    if row < inner_top || row >= border_bottom {
        return None;
    }

    let display_idx = start.checked_add((row - inner_top) as usize)?;
    match rows.get(display_idx)? {
        MenuRow::Lesson(flat_idx) => Some(*flat_idx),
        MenuRow::Header(_) => None,
    }
}

/// Position of the [`MenuRow::Lesson`] row carrying `flat_idx` inside `rows`,
/// or `None` if it is not present.
pub fn display_index_of(rows: &[MenuRow], flat_idx: usize) -> Option<usize> {
    rows.iter()
        .position(|row| matches!(row, MenuRow::Lesson(idx) if *idx == flat_idx))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::Tier;

    fn lesson(id: &str, section_id: &str) -> Lesson {
        Lesson {
            id: id.into(),
            title: id.into(),
            tier: Tier::Tier1Foundation,
            section_id: section_id.into(),
            description: String::new(),
            text: String::new(),
            target_cpm: 0.0,
            min_accuracy: 0.0,
        }
    }

    fn section(id: &str, title: &str) -> Section {
        Section {
            id: id.into(),
            title: title.into(),
            tier: Tier::Tier1Foundation,
            description: String::new(),
        }
    }

    #[test]
    fn test_build_rows_inserts_section_headers() {
        let sections = vec![
            section("fila-guia", "Fila guía"),
            section("fila-superior", "Fila superior"),
        ];
        let l0 = lesson("l0", "fila-guia");
        let l1 = lesson("l1", "fila-guia");
        let l2 = lesson("l2", "fila-superior");
        let tier_lessons: Vec<(usize, &Lesson)> = vec![(5, &l0), (6, &l1), (7, &l2)];

        let rows = build_rows(&tier_lessons, &sections);

        assert_eq!(
            rows,
            vec![
                MenuRow::Header("Fila guía".to_string()),
                MenuRow::Lesson(5),
                MenuRow::Lesson(6),
                MenuRow::Header("Fila superior".to_string()),
                MenuRow::Lesson(7),
            ]
        );
    }

    #[test]
    fn test_build_rows_single_section_has_one_header() {
        let sections = vec![section("fila-guia", "Fila guía")];
        let l0 = lesson("l0", "fila-guia");
        let l1 = lesson("l1", "fila-guia");
        let tier_lessons: Vec<(usize, &Lesson)> = vec![(0, &l0), (1, &l1)];

        let rows = build_rows(&tier_lessons, &sections);

        assert_eq!(
            rows,
            vec![
                MenuRow::Header("Fila guía".to_string()),
                MenuRow::Lesson(0),
                MenuRow::Lesson(1),
            ]
        );
    }

    /// Legacy sliding-window algorithm (`viewport_start`, deleted), ported
    /// verbatim into this test as the equivalence oracle.
    fn legacy_viewport_start(selected_display_idx: usize, capacity: usize) -> usize {
        if capacity == 0 {
            return 0;
        }
        if selected_display_idx >= capacity {
            selected_display_idx + 1 - capacity
        } else {
            0
        }
    }

    #[test]
    fn band_viewport_start_matches_legacy_viewport_start_for_all_inputs() {
        for selected in 0..=9usize {
            for capacity in 0..=7usize {
                assert_eq!(
                    band_viewport_start(selected, capacity),
                    legacy_viewport_start(selected, capacity),
                    "selected={selected} capacity={capacity}"
                );
            }
        }
    }

    #[test]
    fn band_viewport_start_camera_follows_selection() {
        // Selection past the window edge slides the window so the selection
        // stays visible at its trailing edge.
        assert_eq!(band_viewport_start(6, 4), 3);
        assert_eq!(band_viewport_start(5, 4), 2);
        assert_eq!(band_viewport_start(7, 7), 1);
    }

    #[test]
    fn band_viewport_start_in_window_keeps_start() {
        // Selection already inside the window keeps the window pinned at 0
        // (or wherever it was): no panning while the selection is visible.
        assert_eq!(band_viewport_start(0, 4), 0);
        assert_eq!(band_viewport_start(2, 4), 0);
        assert_eq!(band_viewport_start(3, 4), 0);
        assert_eq!(
            band_viewport_start(4, 4),
            1,
            "first index past the window pans by one"
        );
    }

    #[test]
    fn test_lesson_row_at_maps_position() {
        let rows = vec![
            MenuRow::Header("Fila guía".to_string()),
            MenuRow::Lesson(0),
            MenuRow::Lesson(1),
            MenuRow::Header("Fila superior".to_string()),
            MenuRow::Lesson(2),
        ];
        let list_area = Rect::new(0, 0, 20, 10);

        assert_eq!(lesson_row_at(list_area, 0, &rows, 0), None, "top border");
        assert_eq!(lesson_row_at(list_area, 0, &rows, 1), None, "header row");
        assert_eq!(
            lesson_row_at(list_area, 0, &rows, 2),
            Some(0),
            "first lesson row"
        );
        assert_eq!(
            lesson_row_at(list_area, 0, &rows, 3),
            Some(1),
            "second lesson row"
        );
        assert_eq!(lesson_row_at(list_area, 0, &rows, 9), None, "bottom border");
        assert_eq!(lesson_row_at(list_area, 0, &rows, 20), None, "far outside");
    }

    #[test]
    fn test_display_index_of() {
        let rows = vec![
            MenuRow::Header("Fila guía".to_string()),
            MenuRow::Lesson(0),
            MenuRow::Lesson(1),
            MenuRow::Header("Fila superior".to_string()),
            MenuRow::Lesson(2),
        ];

        assert_eq!(display_index_of(&rows, 1), Some(2));
        assert_eq!(display_index_of(&rows, 2), Some(4));
        assert_eq!(display_index_of(&rows, 99), None);
    }

    /// Wide Full-mode area where all 7 planets fit without panning.
    fn wide_full_area() -> Rect {
        Rect::new(0, 0, 200, 40)
    }

    /// Map-body-sized Full-mode area (62% of a 120-col window): capacity 4.
    fn band_full_area() -> Rect {
        Rect::new(0, 0, 74, 31)
    }

    #[test]
    fn planet_layout_returns_exactly_seven_rects() {
        for area in [wide_full_area(), band_full_area(), Rect::new(0, 0, 40, 12)] {
            assert_eq!(planet_layout(area, 0).len(), PLANET_COUNT, "area={area:?}");
        }
    }

    #[test]
    fn planet_layout_orders_planets_left_to_right() {
        // All 7 visible (no panning): x must be strictly increasing Tier1→7.
        let rects = planet_layout(wide_full_area(), 0);
        for w in rects.windows(2) {
            assert!(
                w[1].x > w[0].x,
                "rects must be ordered left-to-right without overlap: {rects:?}"
            );
            assert!(
                w[1].x >= w[0].x + w[0].width,
                "neighbor cards must not overlap: {rects:?}"
            );
        }
        // Panned window keeps the same ordering for the visible planets.
        let panned = planet_layout(band_full_area(), 6);
        let xs: Vec<u16> = panned.iter().filter(|r| r.width > 0).map(|r| r.x).collect();
        assert_eq!(xs.len(), 4, "capacity at 74 cols is 4: {panned:?}");
        for w in xs.windows(2) {
            assert!(w[1] > w[0], "panned window must stay ordered: {panned:?}");
        }
    }

    #[test]
    fn planet_layout_fixed_card_step() {
        let origin = Rect::new(5, 7, 200, 30);
        let rects = planet_layout(origin, 0);
        let step = PLANET_CARD_WIDTH + PLANET_GAP;
        for (i, rect) in rects.iter().enumerate() {
            assert_eq!(rect.x, origin.x + i as u16 * step, "card {i}");
            assert_eq!(
                rect.y,
                origin.y + SHIP_GUTTER_HEIGHT,
                "card {i} sits below the gutter"
            );
            assert_eq!(rect.width, PLANET_CARD_WIDTH, "card {i}");
            assert_eq!(rect.height, PLANET_CARD_HEIGHT, "card {i}");
        }
    }

    #[test]
    fn planet_layout_off_window_planets_are_zero_rects() {
        // 74 cols (a 120-col window's map body) → capacity 4: selecting
        // planet 6 pans the window to 3..=6, so planets 0..=2 collapse to
        // zero rects.
        let area = Rect::new(0, 0, 74, 31);
        let rects = planet_layout(area, 6);
        for (i, rect) in rects.iter().enumerate() {
            let visible = (3..=6).contains(&i);
            assert_eq!(
                rect.width > 0 && rect.height > 0,
                visible,
                "planet {i} visibility mismatch: {rect:?}"
            );
        }
        assert_eq!(rects[6].x, area.x + 3 * (PLANET_CARD_WIDTH + PLANET_GAP));
    }

    #[test]
    fn planet_layout_reserves_top_ship_gutter_row() {
        let area = Rect::new(3, 2, 200, 30);
        let gutter = ship_gutter(area);
        assert_eq!(
            gutter,
            Rect::new(area.x, area.y, area.width, SHIP_GUTTER_HEIGHT)
        );
        for (i, rect) in planet_layout(area, 0).iter().enumerate() {
            assert!(
                rect.y >= area.y + SHIP_GUTTER_HEIGHT,
                "card {i} must start below the ship gutter row: {rect:?}"
            );
        }
    }

    #[test]
    fn planet_layout_compact_mode_seven_horizontal_columns() {
        let area = Rect::new(0, 0, 40, 12);
        assert_eq!(map_mode(area), MapMode::Compact);
        let w = area.width / PLANET_COUNT as u16;
        for selected in [0usize, 6] {
            let rects = planet_layout(area, selected);
            for (i, rect) in rects.iter().enumerate() {
                assert_eq!(
                    rect.x,
                    area.x + i as u16 * w,
                    "column {i} (selected={selected})"
                );
                assert_eq!(rect.y, area.y, "compact columns share the single row");
                assert_eq!(rect.width, w, "7 equal columns (selected={selected})");
                assert_eq!(rect.height, 1, "compact columns are one row tall");
            }
        }
    }

    #[test]
    fn planet_layout_zero_capacity_returns_all_zero_rects() {
        // Compact area narrower than 7 columns: every column width floors to
        // zero, so no planet can render and nothing reports a fake hit rect.
        let area = Rect::new(0, 0, 5, 12);
        assert_eq!(map_mode(area), MapMode::Compact);
        let rects = planet_layout(area, 0);
        assert_eq!(rects.len(), PLANET_COUNT);
        for (i, rect) in rects.iter().enumerate() {
            assert_eq!(
                rect.width * rect.height,
                0,
                "rect {i} must be zero: {rect:?}"
            );
        }
    }

    #[test]
    fn planet_layout_degenerate_areas_never_panic() {
        for area in [
            Rect::new(0, 0, 0, 0),
            Rect::new(0, 0, 1, 1),
            Rect::new(0, 0, 2, 1),
            Rect::new(0, 0, 1, 2),
            Rect::new(4, 3, 6, 2),
        ] {
            let rects = planet_layout(area, 3);
            assert_eq!(rects.len(), PLANET_COUNT, "area={area:?}");
        }
    }

    #[test]
    fn test_map_mode_boundary() {
        // Full mode needs both the minimum width and the derived minimum
        // height (ship gutter row + one card row block).
        assert_eq!(
            map_mode(Rect::new(0, 0, MIN_FULL_WIDTH - 1, MIN_FULL_HEIGHT)),
            MapMode::Compact
        );
        assert_eq!(
            map_mode(Rect::new(0, 0, MIN_FULL_WIDTH, MIN_FULL_HEIGHT - 1)),
            MapMode::Compact
        );
        assert_eq!(
            map_mode(Rect::new(0, 0, MIN_FULL_WIDTH, MIN_FULL_HEIGHT)),
            MapMode::Full
        );
    }

    #[test]
    fn planet_at_hits_only_visible_planets() {
        let area = Rect::new(0, 0, 120, 40);
        let cards = planet_layout(area, 0);
        for (i, card) in cards.iter().enumerate() {
            if card.width == 0 || card.height == 0 {
                continue;
            }
            let center = Position {
                x: card.x + card.width / 2,
                y: card.y + card.height / 2,
            };
            assert_eq!(
                planet_at(area, center, 0),
                Some(i),
                "area={area:?} card {i}={card:?} center={center:?}"
            );
        }
    }

    #[test]
    fn planet_at_requires_selected_for_window() {
        // The same physical point resolves to planet 2 when the window starts
        // at 0 (selected=0), but to planet 5 when selected=6 pans the window
        // to 3..=6 and planet 5 takes over that slot: hit-testing must follow
        // the selected-driven window.
        let area = Rect::new(0, 0, 74, 31);
        let card2 = planet_layout(area, 0)[2];
        let point = Position {
            x: card2.x + card2.width / 2,
            y: card2.y + card2.height / 2,
        };
        assert_eq!(planet_at(area, point, 0), Some(2));
        assert_eq!(
            planet_at(area, point, 6),
            Some(5),
            "card 5 slides into card 2's old slot under selected=6"
        );
    }

    #[test]
    fn test_planet_at_outside_returns_none() {
        let area = Rect::new(0, 0, 120, 40);
        let gutter_point = Position { x: 60, y: 1 };
        assert_eq!(
            planet_at(area, gutter_point, 0),
            None,
            "ship gutter row is not a card"
        );
        let below_all_cards = Position { x: 60, y: 39 };
        assert_eq!(
            planet_at(area, below_all_cards, 0),
            None,
            "below the card band"
        );
    }

    #[test]
    fn ship_gutter_is_top_row_in_full_and_zero_in_compact() {
        let full = Rect::new(0, 0, 120, 40);
        assert_eq!(map_mode(full), MapMode::Full);
        assert_eq!(
            ship_gutter(full),
            Rect::new(full.x, full.y, full.width, SHIP_GUTTER_HEIGHT),
            "Full mode reserves the top rows for the ship lane"
        );
        let compact = Rect::new(0, 0, 40, 12);
        assert_eq!(map_mode(compact), MapMode::Compact);
        assert_eq!(ship_gutter(compact).height, 0, "Compact has no gutter row");
    }

    #[test]
    fn ship_y_parks_at_sprite_lane_core_when_idle() {
        let area = Rect::new(0, 0, 120, 40);
        let gutter = ship_gutter(area);
        let cards = planet_layout(area, 3);
        let core = gutter.y as f32 + gutter.height as f32 / 2.0;
        for phase in [ShipPhase::Idle, ShipPhase::Traveling] {
            let y = ship_y(gutter, &cards, 3, phase, 0.0);
            assert!(
                (y - core).abs() < 1e-4,
                "phase {phase:?} must park at the sprite-lane middle row ({core}), got {y}"
            );
        }
    }

    #[test]
    fn ship_y_interpolates_lane_entry_to_core() {
        let area = Rect::new(0, 0, 120, 40);
        let gutter = ship_gutter(area);
        let cards = planet_layout(area, 3);
        let core = gutter.y as f32 + gutter.height as f32 / 2.0;
        let entry = cards[3].y as f32;
        assert!(entry > core, "docked entry sits below the flight lane");

        // Descending: flight lane → card top as dock_depth goes 0→1.
        assert!((ship_y(gutter, &cards, 3, ShipPhase::Descending, 0.0) - core).abs() < 1e-4);
        let mid = ship_y(gutter, &cards, 3, ShipPhase::Descending, 0.5);
        assert!((mid - (core + (entry - core) * 0.5)).abs() < 1e-4);
        assert!((ship_y(gutter, &cards, 3, ShipPhase::Descending, 1.0) - entry).abs() < 1e-4);

        // Ascending reverses it: card top → flight lane as dock_depth goes 0→1.
        assert!((ship_y(gutter, &cards, 3, ShipPhase::Ascending, 0.0) - entry).abs() < 1e-4);
        assert!((ship_y(gutter, &cards, 3, ShipPhase::Ascending, 1.0) - core).abs() < 1e-4);
    }

    #[test]
    fn ship_x_lerps_card_center_x_by_position() {
        let area = Rect::new(0, 0, 120, 40);
        let cards = planet_layout(area, 0);
        let center_x = |k: usize| cards[k].x as f32 + cards[k].width as f32 / 2.0;

        // Integer positions park exactly on that card's center column.
        for k in [0usize, 3, 6] {
            assert!(
                (ship_x(&cards, k as f32) - center_x(k)).abs() < 1e-4,
                "park at card {k}"
            );
        }

        // Fractional positions interpolate between the straddled centers.
        let half = ship_x(&cards, 0.5);
        assert!((half - (center_x(0) + center_x(1)) / 2.0).abs() < 1e-4);
        let quarter = ship_x(&cards, 3.25);
        assert!(
            (quarter - (center_x(3) + (center_x(4) - center_x(3)) * 0.25)).abs() < 1e-4,
            "pos 3.25 lerps cards 3→4"
        );

        // Out-of-window cards are zero rects: the flight clamps into the
        // visible window instead of lerping toward a fake center at 0.
        let panned = planet_layout(band_full_area(), 6);
        let x = ship_x(&panned, 2.5);
        let window_start_x = panned[3].x as f32 + panned[3].width as f32 / 2.0;
        assert!(
            (x - window_start_x).abs() < 1e-4,
            "pos 2.5 clamps to the first visible card center, got {x}"
        );

        // Degenerate: no visible cards → finite fallback, never a panic.
        let empty: Vec<Rect> = Vec::new();
        assert!(ship_x(&empty, 2.5).is_finite());
    }

    #[test]
    fn test_sprite_lane_info_column_partition() {
        // D8 stacked anatomy: the sprite lane is the top rows of the card,
        // the info lines sit below it across the full card width. The lane
        // is 7 rows tall (the ray-cast sphere disc diameter), so a full
        // card is 10 rows: 7 lane + 3 info.
        let card = Rect::new(0, 0, 14, PLANET_CARD_HEIGHT);
        assert_eq!(card.height, 10, "D8 card anatomy: 7-row lane + 3 info rows");
        assert_eq!(sprite_lane(card), Rect::new(0, 0, 12, 7));
        assert_eq!(info_column(card), Rect::new(0, 7, 14, 3));
        // The two blocks must tile the card exactly: no overlap, no gap.
        assert_eq!(
            sprite_lane(card).y + sprite_lane(card).height,
            info_column(card).y
        );
        assert_eq!(
            info_column(card).y + info_column(card).height,
            card.y + card.height,
            "sprite lane ∪ info column must cover the whole card"
        );

        // Off-origin cards keep the partition anchored to the card rect.
        let off_card = Rect::new(5, 7, 14, PLANET_CARD_HEIGHT);
        assert_eq!(sprite_lane(off_card), Rect::new(5, 7, 12, 7));
        assert_eq!(info_column(off_card), Rect::new(5, 14, 14, 3));

        // Shorter than the lane: the lane clamps, info degrades to a
        // zero-height slice — still no overlap and no gap.
        let short_card = Rect::new(3, 1, 14, 2);
        assert_eq!(sprite_lane(short_card), Rect::new(3, 1, 12, 2));
        assert_eq!(info_column(short_card), Rect::new(3, 3, 14, 0));
        assert_eq!(
            sprite_lane(short_card).y + sprite_lane(short_card).height,
            info_column(short_card).y
        );

        // Mid-height card (taller than the old 3-row lane, shorter than
        // the 7-row lane): the lane still clamps to the card and the info
        // slice stays zero-height — the tiling invariant degrades safely.
        let mid_card = Rect::new(3, 1, 14, 5);
        assert_eq!(sprite_lane(mid_card), Rect::new(3, 1, 12, 5));
        assert_eq!(info_column(mid_card), Rect::new(3, 6, 14, 0));
        assert_eq!(
            sprite_lane(mid_card).y + sprite_lane(mid_card).height,
            info_column(mid_card).y
        );
    }
}
