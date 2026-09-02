//! Pure layout geometry for the tier-map "galaxy" planet view.
//!
//! This module contains no rendering logic (no `Frame`/`Widget`) and no
//! dependency on [`crate::tui::app::App`]; it only computes [`Rect`]s from
//! an input area so the layout math can be unit tested in isolation.

use crate::core::model::{Lesson, Section};
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};

/// Minimum terminal width (in columns) required to render the full galaxy
/// map with a dedicated ship gutter.
pub const MIN_FULL_WIDTH: u16 = 64;
/// Minimum terminal height (in rows) required to render the full galaxy
/// map with a dedicated ship gutter.
pub const MIN_FULL_HEIGHT: u16 = 30;
/// Height (in rows) of a single planet card in [`MapMode::Full`].
pub const PLANET_CARD_HEIGHT: u16 = 3;
/// Width (in columns) of the ship gutter column in [`MapMode::Full`].
pub const SHIP_GUTTER_WIDTH: u16 = 8;
/// Number of tiers/planets rendered on the galaxy map.
pub const PLANET_COUNT: usize = 7;

/// Rendering density for the galaxy map, selected from the available area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapMode {
    /// Roomy layout: card-height planets plus a dedicated ship gutter.
    Full,
    /// Cramped layout: single-row planets, no gutter.
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

/// Computes the area available for the 7 planet cards, i.e. `area` minus
/// the ship gutter column in [`MapMode::Full`].
fn cards_area(area: Rect) -> Rect {
    match map_mode(area) {
        MapMode::Full => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(SHIP_GUTTER_WIDTH), Constraint::Min(0)])
                .split(area);
            chunks[1]
        }
        MapMode::Compact => area,
    }
}

/// Always returns exactly [`PLANET_COUNT`] rects, ordered Tier1..Tier7
/// top-to-bottom. Never panics, even on degenerate (tiny or zero) areas.
pub fn planet_layout(area: Rect) -> Vec<Rect> {
    let card_height = match map_mode(area) {
        MapMode::Full => PLANET_CARD_HEIGHT,
        MapMode::Compact => 1,
    };

    let mut constraints: Vec<Constraint> = (0..PLANET_COUNT)
        .map(|_| Constraint::Length(card_height))
        .collect();
    constraints.push(Constraint::Min(0));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(cards_area(area));

    chunks[0..PLANET_COUNT].to_vec()
}

/// The ship gutter column in [`MapMode::Full`]; a zero-width [`Rect`] in
/// [`MapMode::Compact`].
pub fn ship_gutter(area: Rect) -> Rect {
    match map_mode(area) {
        MapMode::Full => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(SHIP_GUTTER_WIDTH), Constraint::Min(0)])
                .split(area);
            chunks[0]
        }
        MapMode::Compact => Rect::new(area.x, area.y, 0, area.height),
    }
}

/// Hit-tests `pos` against the current planet layout for `area`. Returns
/// the index of the card containing `pos`, or `None` for the gutter or any
/// point outside all cards.
pub fn planet_at(area: Rect, pos: Position) -> Option<usize> {
    planet_layout(area)
        .iter()
        .position(|card| card.width > 0 && card.height > 0 && card.contains(pos))
}

fn center_y(rect: Rect) -> f32 {
    rect.y as f32 + rect.height as f32 / 2.0
}

/// Ship sprite placement stub (finalized in PR6). Interpolates the sprite's
/// vertical center between the cards straddling `pos` (a fractional planet
/// index) and clamps the result inside `gutter`.
pub fn ship_rect(gutter: Rect, cards: &[Rect], pos: f32, sprite_h: u16) -> Rect {
    if cards.is_empty() || gutter.width == 0 || gutter.height == 0 {
        return Rect::new(gutter.x, gutter.y, 0, 0);
    }

    let max_index = cards.len() - 1;
    let i = pos.floor().clamp(0.0, max_index as f32) as usize;
    let t = pos.fract().clamp(0.0, 1.0);
    let next = (i + 1).min(max_index);

    let y0 = center_y(cards[i]);
    let y1 = center_y(cards[next]);
    let lerped_y = y0 + (y1 - y0) * t;
    let sprite_y = lerped_y - sprite_h as f32 / 2.0;

    let clamped_height = sprite_h.min(gutter.height);
    let gutter_bottom = gutter.y.saturating_add(gutter.height);
    let max_top = gutter_bottom.saturating_sub(clamped_height).max(gutter.y);

    let candidate_y = if sprite_y.is_finite() && sprite_y > 0.0 {
        sprite_y.round() as u16
    } else {
        0
    };
    let final_y = candidate_y.clamp(gutter.y, max_top);

    Rect::new(gutter.x, final_y, gutter.width, clamped_height)
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

/// Computes the first visible display row so `selected_display_idx` stays
/// inside a window of `capacity` rows. Matches the pre-PR3 `ui.rs` viewport
/// algorithm. `capacity == 0` always returns `0` (nothing is visible).
pub fn viewport_start(selected_display_idx: usize, capacity: usize) -> usize {
    if capacity == 0 {
        return 0;
    }
    if selected_display_idx >= capacity {
        selected_display_idx + 1 - capacity
    } else {
        0
    }
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

    #[test]
    fn test_viewport_start_matches_prior_algorithm() {
        assert_eq!(viewport_start(0, 5), 0);
        assert_eq!(viewport_start(7, 5), 3);
        assert_eq!(viewport_start(4, 5), 0);
        assert_eq!(viewport_start(9, 0), 0);
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

    fn non_overlapping_top_to_bottom(rects: &[Rect]) {
        let mut prev_bottom: Option<u16> = None;
        for rect in rects.iter().filter(|r| r.height > 0 && r.width > 0) {
            if let Some(prev_bottom) = prev_bottom {
                assert!(
                    rect.y >= prev_bottom,
                    "rects must be ordered top-to-bottom without overlap: {rect:?}"
                );
            }
            prev_bottom = Some(rect.y + rect.height);
        }
    }

    #[test]
    fn test_planet_layout_returns_7_rects_120x40() {
        let area = Rect::new(0, 0, 120, 40);
        let rects = planet_layout(area);
        assert_eq!(rects.len(), PLANET_COUNT);
        non_overlapping_top_to_bottom(&rects);
    }

    #[test]
    fn test_planet_layout_returns_7_rects_64x30() {
        let area = Rect::new(0, 0, 64, 30);
        let rects = planet_layout(area);
        assert_eq!(rects.len(), PLANET_COUNT);
        non_overlapping_top_to_bottom(&rects);
    }

    #[test]
    fn test_planet_layout_never_panics_20x8() {
        let area = Rect::new(0, 0, 20, 8);
        let rects = planet_layout(area);
        assert_eq!(rects.len(), PLANET_COUNT);
        non_overlapping_top_to_bottom(&rects);
    }

    #[test]
    fn test_planet_layout_never_panics_1x1() {
        let area = Rect::new(0, 0, 1, 1);
        let rects = planet_layout(area);
        assert_eq!(rects.len(), PLANET_COUNT);
        non_overlapping_top_to_bottom(&rects);
    }

    #[test]
    fn test_planet_layout_never_panics_zero() {
        let area = Rect::new(0, 0, 0, 0);
        let rects = planet_layout(area);
        assert_eq!(rects.len(), PLANET_COUNT);
        non_overlapping_top_to_bottom(&rects);
    }

    #[test]
    fn test_map_mode_boundary_63x29_vs_64x30() {
        assert_eq!(map_mode(Rect::new(0, 0, 63, 30)), MapMode::Compact);
        assert_eq!(map_mode(Rect::new(0, 0, 64, 29)), MapMode::Compact);
        assert_eq!(map_mode(Rect::new(0, 0, 64, 30)), MapMode::Full);
    }

    #[test]
    fn test_planet_at_hits_card_centers() {
        for area in [Rect::new(0, 0, 120, 40), Rect::new(0, 0, 40, 12)] {
            let cards = planet_layout(area);
            for (i, card) in cards.iter().enumerate() {
                if card.width == 0 || card.height == 0 {
                    continue;
                }
                let center = Position {
                    x: card.x + card.width / 2,
                    y: card.y + card.height / 2,
                };
                assert_eq!(
                    planet_at(area, center),
                    Some(i),
                    "area={area:?} card {i}={card:?} center={center:?}"
                );
            }
        }
    }

    #[test]
    fn test_planet_at_outside_returns_none() {
        let area = Rect::new(0, 0, 120, 40);
        let gutter_point = Position { x: 1, y: 1 };
        assert_eq!(planet_at(area, gutter_point), None);

        let below_all_cards = Position { x: 60, y: 39 };
        assert_eq!(planet_at(area, below_all_cards), None);
    }

    #[test]
    fn test_ship_gutter_zero_width_in_compact() {
        let area = Rect::new(0, 0, 40, 12);
        assert_eq!(map_mode(area), MapMode::Compact);
        assert_eq!(ship_gutter(area).width, 0);
    }

    #[test]
    fn test_ship_rect_stays_inside_gutter() {
        let area = Rect::new(0, 0, 120, 40);
        let gutter = ship_gutter(area);
        let cards = planet_layout(area);
        let sprite_h = 3u16;

        for pos in [0.0f32, 0.5, 3.5, 6.0] {
            let rect = ship_rect(gutter, &cards, pos, sprite_h);
            assert!(
                rect.y >= gutter.y,
                "pos={pos} rect={rect:?} gutter={gutter:?}"
            );
            assert!(
                rect.y + rect.height <= gutter.y + gutter.height,
                "pos={pos} rect={rect:?} gutter={gutter:?}"
            );
        }

        let empty_cards: Vec<Rect> = Vec::new();
        let rect = ship_rect(gutter, &empty_cards, 0.0, sprite_h);
        assert_eq!(rect.width * rect.height, 0);
    }
}
