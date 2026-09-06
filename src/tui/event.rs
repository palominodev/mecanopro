use crate::tui::app::{App, CurrentView};
use crate::tui::planet_layout::{display_index_of, lesson_row_at, planet_at, viewport_start};
use crate::tui::ui::{lesson_list_area, map_body_area};
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::{Position, Rect};
use std::io;

/// Row count moved by `PageUp`/`PageDown`/`Ctrl+b`/`Ctrl+f`/`Ctrl+u`/`Ctrl+d`
/// inside [`CurrentView::PlanetLessons`].
const PAGE_SIZE: usize = 10;

pub struct EventHandler;

impl EventHandler {
    pub fn handle_event(app: &mut App, area: Rect) -> io::Result<()> {
        if event::poll(app.poll_interval())? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    Self::handle_key(app, key);
                }
                Event::Mouse(mouse) => Self::handle_mouse(app, mouse, area)?,
                _ => {}
            }
        }
        Ok(())
    }

    /// Snaps any in-flight ship animation to its destination instantly,
    /// so an event that actually acts on the map never has to wait for it.
    fn finish_animation(app: &mut App) {
        if !app.ship.is_idle() {
            app.ship.complete();
        }
    }

    /// Handles a mouse event against the last known frame `area`. Only mouse
    /// kinds that actually act on the map (`Down(Left)`, `ScrollUp`,
    /// `ScrollDown`) interrupt an in-flight ship animation; passive kinds
    /// like `Moved`/`Drag`/`Up`/`Down(Right)` are fully ignored.
    fn handle_mouse(app: &mut App, m: MouseEvent, area: Rect) -> io::Result<()> {
        if let MouseEventKind::Down(MouseButton::Left) = m.kind {
            Self::finish_animation(app);
            let pos = Position::new(m.column, m.row);
            match app.current_view {
                CurrentView::MainMenu => {
                    if let Some(i) = planet_at(map_body_area(area), pos) {
                        app.selected_planet_index = i;
                        app.ship.snap_to(i);
                        app.enter_planet_lessons();
                    }
                }
                CurrentView::PlanetLessons => Self::handle_lesson_row_click(app, area, m.row),
                _ => {}
            }
        }

        match m.kind {
            MouseEventKind::ScrollUp => {
                Self::finish_animation(app);
                match app.current_view {
                    CurrentView::MainMenu => app.move_planet_up(),
                    CurrentView::PlanetLessons => app.move_selection_up(),
                    _ => {}
                }
            }
            MouseEventKind::ScrollDown => {
                Self::finish_animation(app);
                match app.current_view {
                    CurrentView::MainMenu => app.move_planet_down(),
                    CurrentView::PlanetLessons => app.move_selection_down(),
                    _ => {}
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Resolves a click at terminal `row` against the lesson list, using the
    /// same [`App::current_tier_rows`] and viewport math as the renderer.
    /// Clicking the already-selected row starts practice; any other lesson
    /// row just moves the selection. Headers/borders/outside are a no-op.
    fn handle_lesson_row_click(app: &mut App, area: Rect, row: u16) {
        let rows = app.current_tier_rows();
        let list_area = lesson_list_area(area);
        let capacity = list_area.height.saturating_sub(2) as usize;
        let selected_display = display_index_of(&rows, app.selected_lesson_index).unwrap_or(0);
        let start = viewport_start(selected_display, capacity);

        if let Some(flat) = lesson_row_at(list_area, start, &rows, row) {
            if flat == app.selected_lesson_index {
                if let Some(lesson) = app.selected_lesson() {
                    app.start_practice(lesson);
                }
            } else {
                app.selected_lesson_index = flat;
            }
        }
    }

    fn handle_key(app: &mut App, key: KeyEvent) {
        // Global Ctrl+C exit
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            app.should_quit = true;
            return;
        }

        // Any keypress interrupts an in-flight ship animation, snapping it
        // to its destination instantly instead of swallowing the key.
        if !app.ship.is_idle() {
            app.ship.complete();
        }
        // The animation just completed, so a pending view switch (dock into
        // a planet lane) commits before dispatch: the key applies under the
        // new view in the same cycle, never swallowed by the dock window.
        app.commit_pending_view();

        match app.current_view {
            CurrentView::MainMenu => match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
                KeyCode::Up | KeyCode::Char('k') => app.move_planet_up(),
                KeyCode::Down | KeyCode::Char('j') => app.move_planet_down(),
                KeyCode::Home | KeyCode::Char('g') => app.planet_home(),
                KeyCode::End | KeyCode::Char('G') => app.planet_end(),
                KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => app.enter_planet_lessons(),
                KeyCode::Char('v') | KeyCode::Char('V') => app.start_dictation(None, None),
                KeyCode::Char('d') | KeyCode::Char('D') => app.start_adaptive_drill(),
                KeyCode::Char('e') | KeyCode::Char('E') => app.current_view = CurrentView::Stats,
                KeyCode::Char('b') | KeyCode::Char('B') => app.current_view = CurrentView::History,
                _ => {}
            },

            CurrentView::Practice => match key.code {
                KeyCode::Esc => app.return_from_session(),
                KeyCode::Tab => app.restart_current_session(),
                KeyCode::Char(ch) => app.handle_key_input(ch),
                _ => {}
            },

            CurrentView::Summary => match key.code {
                KeyCode::Char('r') | KeyCode::Char('R') => app.restart_current_session(),
                KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Enter => app.next_lesson(),
                KeyCode::Esc | KeyCode::Char('m') | KeyCode::Char('M') => app.return_from_session(),
                _ => {}
            },

            CurrentView::Stats => match key.code {
                KeyCode::Char('v') | KeyCode::Char('V') => app.start_dictation(None, None),
                KeyCode::Char('d') | KeyCode::Char('D') => app.start_adaptive_drill(),
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') | KeyCode::Char('Q') => {
                    app.return_to_star_map();
                }
                _ => {}
            },

            // Exit binding wired in task 5.4; entry-only for now (task 5.2).
            CurrentView::History => {}

            CurrentView::Dictation => {
                if key.modifiers.contains(KeyModifiers::CONTROL) && (key.code == KeyCode::Char('v') || key.code == KeyCode::Char('V')) {
                    app.toggle_dictation_voice();
                    return;
                }

                match key.code {
                    KeyCode::Esc => {
                        app.tts_speaker.stop();
                        app.return_to_star_map();
                    }
                    KeyCode::Tab => app.replay_dictation_audio(),
                    KeyCode::BackTab | KeyCode::F(2) => app.toggle_dictation_voice(),
                    KeyCode::Char('+') | KeyCode::Char('=') => app.adjust_dictation_speed(0.1),
                    KeyCode::Char('-') | KeyCode::Char('_') => app.adjust_dictation_speed(-0.1),
                    KeyCode::Char(ch) => app.handle_dictation_key_input(ch),
                    _ => {}
                }
            }

            CurrentView::DictationSummary => match key.code {
                KeyCode::Char('r') | KeyCode::Char('R') => app.restart_dictation(),
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Enter => app.start_dictation(None, None),
                KeyCode::Esc | KeyCode::Char('m') | KeyCode::Char('M') => app.return_to_star_map(),
                _ => {}
            },

            CurrentView::PlanetLessons => {
                let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => app.move_selection_up(),
                    KeyCode::Down | KeyCode::Char('j') => app.move_selection_down(),
                    KeyCode::PageUp => app.scroll_page_up(PAGE_SIZE),
                    KeyCode::PageDown => app.scroll_page_down(PAGE_SIZE),
                    KeyCode::Char('b') if ctrl => app.scroll_page_up(PAGE_SIZE),
                    KeyCode::Char('f') if ctrl => app.scroll_page_down(PAGE_SIZE),
                    KeyCode::Char('u') if ctrl => app.scroll_page_up(PAGE_SIZE),
                    KeyCode::Char('d') if ctrl => app.scroll_page_down(PAGE_SIZE),
                    KeyCode::Home | KeyCode::Char('g') => app.scroll_to_top(),
                    KeyCode::End | KeyCode::Char('G') => app.scroll_to_bottom(),
                    KeyCode::Enter => {
                        if let Some(lesson) = app.selected_lesson() {
                            app.start_practice(lesson);
                        }
                    }
                    KeyCode::Esc
                    | KeyCode::Backspace
                    | KeyCode::Left
                    | KeyCode::Char('h')
                    | KeyCode::Char('q') => app.leave_planet_lessons(),
                    _ => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::app::App;
    use crate::tui::planet_layout::{planet_layout, MenuRow};
    use crate::core::model::{Tier, UserProgress};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn mouse(kind: MouseEventKind, pos: Position) -> MouseEvent {
        MouseEvent {
            kind,
            column: pos.x,
            row: pos.y,
            modifiers: KeyModifiers::NONE,
        }
    }

    fn left_click(pos: Position) -> MouseEvent {
        mouse(MouseEventKind::Down(MouseButton::Left), pos)
    }

    #[test]
    fn test_left_click_on_planet_card_selects_and_opens_planetlessons() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        let area = Rect::new(0, 0, 120, 40);
        let cards = planet_layout(map_body_area(area));
        let card = cards[3];
        let pos = Position::new(card.x + card.width / 2, card.y + card.height / 2);

        EventHandler::handle_mouse(&mut app, left_click(pos), area).unwrap();
        // Mouse never commits the pending view itself: complete the dock so
        // the deferred PlanetLessons switch goes through before asserting.
        app.advance_animation(crate::tui::animation::DESCEND + std::time::Duration::from_millis(1));

        assert_eq!(app.selected_planet_index, 3);
        assert_eq!(app.current_view, CurrentView::PlanetLessons);
    }

    #[test]
    fn test_left_click_outside_ignored() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier2FullAlphabet.index();
        let area = Rect::new(0, 0, 120, 40);

        EventHandler::handle_mouse(&mut app, left_click(Position::new(0, 0)), area).unwrap();

        assert_eq!(app.current_view, CurrentView::MainMenu);
        assert_eq!(app.selected_planet_index, Tier::Tier2FullAlphabet.index());
    }

    #[test]
    fn test_left_click_on_lesson_row_selects() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        app.enter_planet_lessons();
        // Deferred view switch: complete the dock first, so the click below
        // dispatches under PlanetLessons (lesson rows), not MainMenu.
        app.advance_animation(crate::tui::animation::DESCEND + std::time::Duration::from_millis(1));
        let area = Rect::new(0, 0, 120, 40);
        let list_area = lesson_list_area(area);
        let rows = app.current_tier_rows();
        // Second `MenuRow::Lesson` row: the tier's first section starts with
        // a header, so display row 2 is the second lesson (row 1 is the
        // first lesson right after the header at row 0).
        let flat = match rows[2] {
            MenuRow::Lesson(idx) => idx,
            _ => panic!("expected rows[2] to be a Lesson row: {:?}", rows[2]),
        };
        let row_y = list_area.y + 1 + 2; // inner_top + display index

        EventHandler::handle_mouse(&mut app, left_click(Position::new(list_area.x + 2, row_y)), area).unwrap();

        assert_eq!(app.selected_lesson_index, flat);
        assert_eq!(app.current_view, CurrentView::PlanetLessons);
    }

    #[test]
    fn test_left_click_on_already_selected_row_starts_practice() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        app.enter_planet_lessons();
        // Deferred view switch: complete the dock first, so the click below
        // dispatches under PlanetLessons (lesson rows), not MainMenu.
        app.advance_animation(crate::tui::animation::DESCEND + std::time::Duration::from_millis(1));
        let area = Rect::new(0, 0, 120, 40);
        let list_area = lesson_list_area(area);
        let rows = app.current_tier_rows();
        let selected_display =
            display_index_of(&rows, app.selected_lesson_index).expect("selected lesson must be a row");
        let row_y = list_area.y + 1 + selected_display as u16;

        EventHandler::handle_mouse(&mut app, left_click(Position::new(list_area.x + 2, row_y)), area).unwrap();

        assert_eq!(app.current_view, CurrentView::Practice);
    }

    #[test]
    fn test_click_during_animation_completes_it_first() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;
        app.move_planet_down();
        assert_eq!(app.ship.phase(), crate::tui::animation::ShipPhase::Traveling);

        let area = Rect::new(0, 0, 120, 40);
        let cards = planet_layout(map_body_area(area));
        let card = cards[3];
        let pos = Position::new(card.x + card.width / 2, card.y + card.height / 2);
        EventHandler::handle_mouse(&mut app, left_click(pos), area).unwrap();
        // Mouse never commits the pending view itself: complete the dock so
        // the deferred PlanetLessons switch goes through before asserting.
        app.advance_animation(crate::tui::animation::DESCEND + std::time::Duration::from_millis(1));

        assert_ne!(app.ship.phase(), crate::tui::animation::ShipPhase::Traveling);
        assert!((app.ship.position() - 3.0).abs() < 1e-5);
        assert_eq!(app.current_view, CurrentView::PlanetLessons);
    }

    #[test]
    fn test_mouse_motion_does_not_complete_animation() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;
        app.move_planet_down();
        assert_eq!(app.ship.phase(), crate::tui::animation::ShipPhase::Traveling);
        let position_before = app.ship.position();

        let area = Rect::new(0, 0, 120, 40);
        for kind in [
            MouseEventKind::Moved,
            MouseEventKind::Drag(MouseButton::Left),
            MouseEventKind::Up(MouseButton::Left),
            MouseEventKind::Down(MouseButton::Right),
        ] {
            EventHandler::handle_mouse(&mut app, mouse(kind, Position::new(0, 0)), area).unwrap();
            assert_eq!(
                app.ship.phase(),
                crate::tui::animation::ShipPhase::Traveling,
                "kind {kind:?} must not complete the in-flight animation"
            );
            assert!(
                (app.ship.position() - position_before).abs() < 1e-5,
                "kind {kind:?} must not change ship position"
            );
        }
    }

    #[test]
    fn test_non_click_mouse_kinds_ignored() {
        let area = Rect::new(0, 0, 120, 40);
        let cards = planet_layout(map_body_area(area));
        let card = cards[3];
        let pos = Position::new(card.x + card.width / 2, card.y + card.height / 2);

        for kind in [
            MouseEventKind::Moved,
            MouseEventKind::Drag(MouseButton::Left),
            MouseEventKind::Up(MouseButton::Left),
            MouseEventKind::Down(MouseButton::Right),
        ] {
            let mut app = App::new();
            app.user_progress = UserProgress::default();
            app.selected_planet_index = Tier::Tier2FullAlphabet.index();

            EventHandler::handle_mouse(&mut app, mouse(kind, pos), area).unwrap();

            assert_eq!(app.current_view, CurrentView::MainMenu, "kind {kind:?} must not change view");
            assert_eq!(
                app.selected_planet_index,
                Tier::Tier2FullAlphabet.index(),
                "kind {kind:?} must not change selection"
            );
        }
    }

    #[test]
    fn test_scroll_moves_selection_both_views() {
        let area = Rect::new(0, 0, 120, 40);

        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier2FullAlphabet.index();
        EventHandler::handle_mouse(&mut app, mouse(MouseEventKind::ScrollDown, Position::new(0, 0)), area).unwrap();
        assert_eq!(app.selected_planet_index, Tier::Tier3SpanishOrthography.index());

        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        app.enter_planet_lessons();
        // Deferred view switch: complete the dock so the scroll below acts on
        // the lesson list (MainMenu scroll would move the planet selection).
        app.advance_animation(crate::tui::animation::DESCEND + std::time::Duration::from_millis(1));
        app.move_selection_down();
        let before = app.selected_lesson_index;
        EventHandler::handle_mouse(&mut app, mouse(MouseEventKind::ScrollUp, Position::new(0, 0)), area).unwrap();
        assert_eq!(app.selected_lesson_index, before - 1);
    }

    #[test]
    fn test_left_click_on_header_row_noop() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        app.enter_planet_lessons();
        // Deferred view switch: complete the dock so the click below hits the
        // lesson row under PlanetLessons semantics.
        app.advance_animation(crate::tui::animation::DESCEND + std::time::Duration::from_millis(1));
        let before_lesson = app.selected_lesson_index;
        let area = Rect::new(0, 0, 120, 40);
        let list_area = lesson_list_area(area);
        let header_row_y = list_area.y + 1; // display row 0 is the section header

        EventHandler::handle_mouse(&mut app, left_click(Position::new(list_area.x + 2, header_row_y)), area).unwrap();

        assert_eq!(app.current_view, CurrentView::PlanetLessons);
        assert_eq!(app.selected_lesson_index, before_lesson);
    }

    #[test]
    fn test_esc_backspace_left_h_q_preserve_selected_planet() {
        for code in [KeyCode::Esc, KeyCode::Backspace, KeyCode::Left, KeyCode::Char('h'), KeyCode::Char('q')] {
            let mut app = App::new();
            app.user_progress = UserProgress::default();
            app.selected_planet_index = Tier::Tier3SpanishOrthography.index();
            app.enter_planet_lessons();

            EventHandler::handle_key(&mut app, key(code));

            assert_eq!(app.current_view, CurrentView::MainMenu, "key {code:?} must return to MainMenu");
            assert_eq!(
                app.selected_planet_index,
                Tier::Tier3SpanishOrthography.index(),
                "key {code:?} must preserve selected_planet_index"
            );
        }
    }

    #[test]
    fn test_mainmenu_enter_opens_planetlessons() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier2FullAlphabet.index();

        EventHandler::handle_key(&mut app, key(KeyCode::Enter));
        // Deferred view switch: complete the dock so the pending view commits.
        app.advance_animation(crate::tui::animation::DESCEND + std::time::Duration::from_millis(1));

        assert_eq!(app.current_view, CurrentView::PlanetLessons);
        let selected = app.selected_lesson().expect("a lesson must be selected");
        assert_eq!(selected.tier, Tier::Tier2FullAlphabet);
    }

    #[test]
    fn test_practice_esc_returns_to_planetlessons_of_tier() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        let tier3_lesson = app
            .available_lessons()
            .into_iter()
            .find(|l| l.tier == Tier::Tier3SpanishOrthography)
            .expect("fixture needs a Tier3 lesson");
        app.start_practice(tier3_lesson);
        assert_eq!(app.current_view, CurrentView::Practice);

        EventHandler::handle_key(&mut app, key(KeyCode::Esc));

        assert_eq!(app.current_view, CurrentView::PlanetLessons);
        assert_eq!(app.selected_planet_index, Tier::Tier3SpanishOrthography.index());
    }

    #[test]
    fn test_any_keypress_during_animation_completes_it_first() {
        let mut app = App::new();
        app.selected_planet_index = 0;
        app.ship = crate::tui::animation::ShipAnimation::new(0);
        app.move_planet_down();
        // No time has elapsed yet, so without the skip guard the ship would
        // still be interpolating from 0.0 (its `from`) when the next travel
        // retargets it.
        assert_eq!(app.ship.phase(), crate::tui::animation::ShipPhase::Traveling);
        assert!((app.ship.position() - 0.0).abs() < 1e-5);

        EventHandler::handle_key(&mut app, key(KeyCode::Down));

        // The guard must complete the FIRST travel (snapping to planet 1)
        // before the key's own navigation starts a second travel (toward
        // planet 2); the new travel's `from` proves the snap happened,
        // instead of continuing to interpolate from the stale 0.0 origin.
        assert_eq!(app.ship.phase(), crate::tui::animation::ShipPhase::Traveling);
        assert!(
            (app.ship.position() - 1.0).abs() < 1e-5,
            "expected the completed first hop (1.0) as the new travel's origin, got {}",
            app.ship.position()
        );
        assert_eq!(app.selected_planet_index, 2, "the key itself must still be processed");
    }

    #[test]
    fn test_stats_and_dictation_return_to_star_map() {
        let tier5_lesson = App::new()
            .available_lessons()
            .into_iter()
            .find(|l| l.tier == Tier::Tier5SpeedAndCadence)
            .expect("fixture needs a Tier5 lesson");

        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_lesson_index = app.flat_index_of(&tier5_lesson.id).unwrap();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        app.current_view = CurrentView::Stats;
        EventHandler::handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.current_view, CurrentView::MainMenu);
        assert_eq!(
            app.selected_planet_index,
            Tier::Tier5SpeedAndCadence.index(),
            "Stats back must derive the planet from the selected lesson via return_to_star_map"
        );

        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_lesson_index = app.flat_index_of(&tier5_lesson.id).unwrap();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        app.current_view = CurrentView::Dictation;
        EventHandler::handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.current_view, CurrentView::MainMenu);
        assert_eq!(
            app.selected_planet_index,
            Tier::Tier5SpeedAndCadence.index(),
            "Dictation back must derive the planet from the selected lesson via return_to_star_map"
        );
    }

    /// Expected observable effect of one `MainMenu` key binding.
    #[derive(Debug)]
    enum MenuEffect {
        PlanetIndex(usize),
        View(CurrentView),
        Quit,
    }

    #[test]
    fn test_map_key_bindings() {
        // `d` (adaptive drill) is intentionally skipped: it needs real key
        // stats/audio wiring, already covered indirectly by
        // `test_return_from_session_adaptive_drill_goes_to_star_map` in `app.rs`.
        let cases: Vec<(&str, usize, KeyEvent, MenuEffect)> = vec![
            ("Down", 0, key(KeyCode::Down), MenuEffect::PlanetIndex(1)),
            ("j", 0, key(KeyCode::Char('j')), MenuEffect::PlanetIndex(1)),
            ("Up", 3, key(KeyCode::Up), MenuEffect::PlanetIndex(2)),
            ("k", 3, key(KeyCode::Char('k')), MenuEffect::PlanetIndex(2)),
            ("End", 0, key(KeyCode::End), MenuEffect::PlanetIndex(6)),
            ("G", 0, key(KeyCode::Char('G')), MenuEffect::PlanetIndex(6)),
            ("Home", 4, key(KeyCode::Home), MenuEffect::PlanetIndex(0)),
            ("g", 4, key(KeyCode::Char('g')), MenuEffect::PlanetIndex(0)),
            ("Right", 2, key(KeyCode::Right), MenuEffect::View(CurrentView::PlanetLessons)),
            ("l", 2, key(KeyCode::Char('l')), MenuEffect::View(CurrentView::PlanetLessons)),
            ("e", 0, key(KeyCode::Char('e')), MenuEffect::View(CurrentView::Stats)),
            ("v", 0, key(KeyCode::Char('v')), MenuEffect::View(CurrentView::Dictation)),
            ("b", 0, key(KeyCode::Char('b')), MenuEffect::View(CurrentView::History)),
            ("B", 0, key(KeyCode::Char('B')), MenuEffect::View(CurrentView::History)),
            ("q", 0, key(KeyCode::Char('q')), MenuEffect::Quit),
            ("Ctrl+c", 0, KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL), MenuEffect::Quit),
        ];

        for (label, start_index, evt, effect) in cases {
            let mut app = App::new();
            app.user_progress = UserProgress::default();
            app.selected_planet_index = start_index;

            EventHandler::handle_key(&mut app, evt);
            // Deferred view switch: a uniform advance is safe for every case
            // — it completes in-flight travels (landing the assertions on the
            // snap target) and commits the dock for the Right/l enter cases.
            app.advance_animation(
                crate::tui::animation::DESCEND + std::time::Duration::from_millis(1),
            );

            match effect {
                MenuEffect::PlanetIndex(expected) => assert_eq!(
                    app.selected_planet_index, expected,
                    "key {label} ({evt:?}) must set selected_planet_index to {expected}"
                ),
                MenuEffect::View(expected) => assert_eq!(
                    app.current_view, expected,
                    "key {label} ({evt:?}) must set current_view to {expected:?}"
                ),
                MenuEffect::Quit => assert!(app.should_quit, "key {label} ({evt:?}) must set should_quit"),
            }
        }
    }

    #[test]
    fn test_planet_lessons_key_bindings() {
        fn enter_tier1() -> App {
            let mut app = App::new();
            app.user_progress = UserProgress::default();
            app.selected_planet_index = Tier::Tier1Foundation.index();
            app.enter_planet_lessons();
            app
        }

        let tier1_lessons: Vec<_> = App::new()
            .available_lessons()
            .into_iter()
            .filter(|l| l.tier == Tier::Tier1Foundation)
            .collect();
        let first_idx = App::new().flat_index_of(&tier1_lessons.first().unwrap().id).unwrap();
        let last_idx = App::new().flat_index_of(&tier1_lessons.last().unwrap().id).unwrap();

        // Enter starts practice on the currently selected lesson.
        let mut app = enter_tier1();
        EventHandler::handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.current_view, CurrentView::Practice, "Enter must start practice");

        // PageDown / Ctrl+f / Ctrl+d advance the selection by PAGE_SIZE, clamped to the tier's last lesson.
        for evt in [
            key(KeyCode::PageDown),
            KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
        ] {
            let mut app = enter_tier1();
            app.selected_lesson_index = first_idx;
            EventHandler::handle_key(&mut app, evt);
            assert_eq!(
                app.selected_lesson_index,
                (first_idx + PAGE_SIZE).min(last_idx),
                "key {evt:?} must advance the selection by PAGE_SIZE, clamped"
            );
        }

        // PageUp / Ctrl+b / Ctrl+u move the selection back by PAGE_SIZE, clamped to the tier's first lesson.
        for evt in [
            key(KeyCode::PageUp),
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
        ] {
            let mut app = enter_tier1();
            app.selected_lesson_index = last_idx;
            EventHandler::handle_key(&mut app, evt);
            assert_eq!(
                app.selected_lesson_index,
                last_idx.saturating_sub(PAGE_SIZE).max(first_idx),
                "key {evt:?} must move the selection back by PAGE_SIZE, clamped"
            );
        }

        // End / G jump to the last lesson of the tier.
        for evt in [key(KeyCode::End), key(KeyCode::Char('G'))] {
            let mut app = enter_tier1();
            EventHandler::handle_key(&mut app, evt);
            assert_eq!(app.selected_lesson_index, last_idx, "key {evt:?} must select the tier's last lesson");
        }

        // Home / g jump to the first lesson of the tier.
        for evt in [key(KeyCode::Home), key(KeyCode::Char('g'))] {
            let mut app = enter_tier1();
            app.selected_lesson_index = last_idx;
            EventHandler::handle_key(&mut app, evt);
            assert_eq!(app.selected_lesson_index, first_idx, "key {evt:?} must select the tier's first lesson");
        }
    }
}
