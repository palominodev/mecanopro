use crate::tui::app::{App, CurrentView};
use crate::tui::planet_layout::{band_viewport_start, display_index_of, lesson_row_at, planet_at};
use crate::tui::ui::{lesson_list_area, map_body_area};
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
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

    /// Handles a mouse event against the last known frame `area`. Only mouse
    /// kinds that actually act on the map (`Down(Left)`, `ScrollUp`,
    /// `ScrollDown`) interrupt an in-flight ship animation (routing it
    /// through [`App::finish_ship_animation`]); passive kinds like
    /// `Moved`/`Drag`/`Up`/`Down(Right)` are fully ignored.
    fn handle_mouse(app: &mut App, m: MouseEvent, area: Rect) -> io::Result<()> {
        if let MouseEventKind::Down(MouseButton::Left) = m.kind {
            app.finish_ship_animation();
            let pos = Position::new(m.column, m.row);
            match app.current_view {
                CurrentView::MainMenu => {
                    if let Some(i) = planet_at(map_body_area(area), pos, app.selected_planet_index)
                    {
                        // D11 click part: clicking the docked (selected)
                        // card confirms the dock and opens the
                        // observatory; any other card selects it, snaps
                        // the ship there, and starts a fresh descend.
                        if i == app.selected_planet_index && app.is_docked() {
                            app.confirm_planet();
                        } else {
                            app.selected_planet_index = i;
                            app.ship.snap_to(i);
                            app.docked = false;
                            app.ship.descend();
                        }
                    }
                }
                CurrentView::PlanetLessons => Self::handle_lesson_row_click(app, area, m.row),
                _ => {}
            }
        }

        match m.kind {
            // D11: the wheel scrolls the horizontal planet band — vertical
            // wheel directions map onto left/right moves in MainMenu.
            MouseEventKind::ScrollUp | MouseEventKind::ScrollLeft => {
                app.finish_ship_animation();
                match app.current_view {
                    CurrentView::MainMenu => app.move_planet_left(),
                    CurrentView::PlanetLessons => app.move_selection_up(),
                    _ => {}
                }
            }
            MouseEventKind::ScrollDown | MouseEventKind::ScrollRight => {
                app.finish_ship_animation();
                match app.current_view {
                    CurrentView::MainMenu => app.move_planet_right(),
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
        let start = band_viewport_start(selected_display, capacity);

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

        // Any keypress routes an in-flight ship animation through
        // finish_ship_animation, snapping it to its destination instantly
        // instead of swallowing the key — and docking it when the flight
        // was a descend, so the key below dispatches under the landed
        // state (e.g. a mid-descent Enter opens in the same press).
        app.finish_ship_animation();

        match app.current_view {
            CurrentView::MainMenu => match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
                // D11 nav remap: the band is horizontal, so Left/h and
                // Right/l pan between planets; Right no longer opens the
                // lesson list (Enter keeps that job) and Up/Down/j/k are
                // deliberately unmapped.
                KeyCode::Left | KeyCode::Char('h') => app.move_planet_left(),
                KeyCode::Right | KeyCode::Char('l') => app.move_planet_right(),
                KeyCode::Home | KeyCode::Char('g') => app.planet_home(),
                KeyCode::End | KeyCode::Char('G') => app.planet_end(),
                KeyCode::Enter => app.confirm_planet(),
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

            CurrentView::History => match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') | KeyCode::Char('Q') => {
                    app.return_to_star_map();
                }
                _ => {}
            },

            CurrentView::Dictation => {
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && (key.code == KeyCode::Char('v') || key.code == KeyCode::Char('V'))
                {
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
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Enter => {
                    app.start_dictation(None, None)
                }
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

            // Observatory keymap: Enter opens the docked planet's lesson
            // list; Esc ascends back to the galaxy map (the same
            // leave/ascend semantics PlanetLessons uses). Both exits
            // lift the dock.
            CurrentView::Observatory => match key.code {
                KeyCode::Enter => app.open_planet_lessons(),
                KeyCode::Esc => app.leave_planet_lessons(),
                _ => {}
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::{Tier, UserProgress};
    use crate::tui::app::App;
    use crate::tui::planet_layout::{planet_layout, MenuRow};

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

    /// Three-step Enter (D2): the first press starts the descend; the
    /// second lands inside the dock window and fast-paths — the handler
    /// finishes the dock and the same key dispatches `confirm_planet`,
    /// opening the observatory; the third press opens the lessons from
    /// the observatory. Replaces the old two-step Enter idiom.
    fn open_selected_planet_lessons(app: &mut App) {
        EventHandler::handle_key(app, key(KeyCode::Enter));
        EventHandler::handle_key(app, key(KeyCode::Enter));
        assert_eq!(app.current_view, CurrentView::Observatory);
        EventHandler::handle_key(app, key(KeyCode::Enter));
        assert_eq!(app.current_view, CurrentView::PlanetLessons);
    }

    #[test]
    fn test_left_click_on_planet_card_selects_and_descends() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        let area = Rect::new(0, 0, 120, 40);
        let cards = planet_layout(map_body_area(area), app.selected_planet_index);
        let card = cards[3];
        let pos = Position::new(card.x + card.width / 2, card.y + card.height / 2);

        EventHandler::handle_mouse(&mut app, left_click(pos), area).unwrap();
        // Settle the click's descend so the dock state is observable.
        app.advance_animation(crate::tui::animation::DESCEND + std::time::Duration::from_millis(1));

        assert_eq!(app.selected_planet_index, 3);
        assert_eq!(
            app.current_view,
            CurrentView::MainMenu,
            "a click docks at the card; opening needs a confirm on the docked card"
        );
        assert!(app.is_docked());
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
        // Three-step Enter: the click below must dispatch under PlanetLessons
        // (lesson rows), not MainMenu.
        open_selected_planet_lessons(&mut app);
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

        EventHandler::handle_mouse(
            &mut app,
            left_click(Position::new(list_area.x + 2, row_y)),
            area,
        )
        .unwrap();

        assert_eq!(app.selected_lesson_index, flat);
        assert_eq!(app.current_view, CurrentView::PlanetLessons);
    }

    #[test]
    fn test_left_click_on_already_selected_row_starts_practice() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        // Three-step Enter: the click below must dispatch under PlanetLessons
        // (lesson rows), not MainMenu.
        open_selected_planet_lessons(&mut app);
        let area = Rect::new(0, 0, 120, 40);
        let list_area = lesson_list_area(area);
        let rows = app.current_tier_rows();
        let selected_display = display_index_of(&rows, app.selected_lesson_index)
            .expect("selected lesson must be a row");
        let row_y = list_area.y + 1 + selected_display as u16;

        EventHandler::handle_mouse(
            &mut app,
            left_click(Position::new(list_area.x + 2, row_y)),
            area,
        )
        .unwrap();

        assert_eq!(app.current_view, CurrentView::Practice);
    }

    #[test]
    fn test_click_during_animation_completes_it_first() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;
        app.move_planet_right();
        assert_eq!(
            app.ship.phase(),
            crate::tui::animation::ShipPhase::Traveling
        );

        let area = Rect::new(0, 0, 120, 40);
        let cards = planet_layout(map_body_area(area), 1);
        let card = cards[3];
        let pos = Position::new(card.x + card.width / 2, card.y + card.height / 2);
        EventHandler::handle_mouse(&mut app, left_click(pos), area).unwrap();
        // Settle the click's descend so the final dock state is observable.
        app.advance_animation(crate::tui::animation::DESCEND + std::time::Duration::from_millis(1));

        assert_ne!(
            app.ship.phase(),
            crate::tui::animation::ShipPhase::Traveling
        );
        assert!((app.ship.position() - 3.0).abs() < 1e-5);
        assert_eq!(
            app.current_view,
            CurrentView::MainMenu,
            "the click selects and docks; opening needs a confirm on the docked card"
        );
        assert!(app.is_docked());
    }

    #[test]
    fn test_mouse_motion_does_not_complete_animation() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;
        app.move_planet_right();
        assert_eq!(
            app.ship.phase(),
            crate::tui::animation::ShipPhase::Traveling
        );
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
        let cards = planet_layout(map_body_area(area), Tier::Tier2FullAlphabet.index());
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

            assert_eq!(
                app.current_view,
                CurrentView::MainMenu,
                "kind {kind:?} must not change view"
            );
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
        EventHandler::handle_mouse(
            &mut app,
            mouse(MouseEventKind::ScrollDown, Position::new(0, 0)),
            area,
        )
        .unwrap();
        assert_eq!(
            app.selected_planet_index,
            Tier::Tier3SpanishOrthography.index()
        );

        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        // Three-step Enter: the scroll below must act on the lesson list
        // (MainMenu scroll would move the planet selection).
        open_selected_planet_lessons(&mut app);
        app.move_selection_down();
        let before = app.selected_lesson_index;
        EventHandler::handle_mouse(
            &mut app,
            mouse(MouseEventKind::ScrollUp, Position::new(0, 0)),
            area,
        )
        .unwrap();
        assert_eq!(app.selected_lesson_index, before - 1);
    }

    #[test]
    fn test_left_click_on_header_row_noop() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        // Three-step Enter: the click below must hit the lesson row under
        // PlanetLessons semantics.
        open_selected_planet_lessons(&mut app);
        let before_lesson = app.selected_lesson_index;
        let area = Rect::new(0, 0, 120, 40);
        let list_area = lesson_list_area(area);
        let header_row_y = list_area.y + 1; // display row 0 is the section header

        EventHandler::handle_mouse(
            &mut app,
            left_click(Position::new(list_area.x + 2, header_row_y)),
            area,
        )
        .unwrap();

        assert_eq!(app.current_view, CurrentView::PlanetLessons);
        assert_eq!(app.selected_lesson_index, before_lesson);
    }

    #[test]
    fn test_esc_backspace_left_h_q_preserve_selected_planet() {
        for code in [
            KeyCode::Esc,
            KeyCode::Backspace,
            KeyCode::Left,
            KeyCode::Char('h'),
            KeyCode::Char('q'),
        ] {
            let mut app = App::new();
            app.user_progress = UserProgress::default();
            app.selected_planet_index = Tier::Tier3SpanishOrthography.index();
            // Three-step Enter: the leave keys below must act on an actually
            // open PlanetLessons view.
            open_selected_planet_lessons(&mut app);

            EventHandler::handle_key(&mut app, key(code));

            assert_eq!(
                app.current_view,
                CurrentView::MainMenu,
                "key {code:?} must return to MainMenu"
            );
            assert_eq!(
                app.selected_planet_index,
                Tier::Tier3SpanishOrthography.index(),
                "key {code:?} must preserve selected_planet_index"
            );
        }
    }

    #[test]
    fn test_mainmenu_enter_thrice_opens_observatory_then_lessons() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier2FullAlphabet.index();

        // First Enter: the descend starts and the map stays on screen.
        EventHandler::handle_key(&mut app, key(KeyCode::Enter));
        app.advance_animation(crate::tui::animation::DESCEND + std::time::Duration::from_millis(1));
        assert_eq!(app.current_view, CurrentView::MainMenu);
        assert!(app.is_docked(), "the settled descend docks the ship");

        // Second Enter: the docked ship confirms and the observatory
        // opens — the dock holds (it is the docked view).
        EventHandler::handle_key(&mut app, key(KeyCode::Enter));

        assert_eq!(app.current_view, CurrentView::Observatory);
        assert!(app.is_docked(), "the observatory keeps the dock");

        // Third Enter: the observatory opens the lessons and lifts the
        // dock.
        EventHandler::handle_key(&mut app, key(KeyCode::Enter));

        assert_eq!(app.current_view, CurrentView::PlanetLessons);
        assert!(!app.is_docked(), "opening the lessons lifts the dock");
        let selected = app.selected_lesson().expect("a lesson must be selected");
        assert_eq!(selected.tier, Tier::Tier2FullAlphabet);
    }

    #[test]
    fn test_mainmenu_single_enter_without_settle_stays_mainmenu() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier2FullAlphabet.index();

        EventHandler::handle_key(&mut app, key(KeyCode::Enter));

        assert_eq!(
            app.ship.phase(),
            crate::tui::animation::ShipPhase::Descending
        );
        assert!(!app.is_docked());
        assert_eq!(
            app.current_view,
            CurrentView::MainMenu,
            "one Enter alone must never flip the view"
        );
    }

    #[test]
    fn test_docked_left_lifts_and_travels() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 3;
        app.ship = crate::tui::animation::ShipAnimation::new(3);
        app.confirm_planet();
        app.finish_ship_animation();
        assert!(app.is_docked());

        EventHandler::handle_key(&mut app, key(KeyCode::Left));

        assert_eq!(app.selected_planet_index, 2);
        assert!(!app.is_docked(), "an actual travel must lift the dock");
        assert_eq!(
            app.ship.phase(),
            crate::tui::animation::ShipPhase::Traveling
        );
    }

    #[test]
    fn test_keys_never_swallowed_during_dock_window() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier2FullAlphabet.index();
        app.ship = crate::tui::animation::ShipAnimation::new(app.selected_planet_index);

        // Mid-descent (inside the dock window): a single Enter must BOTH
        // finish the dock and dispatch — one press opens the observatory,
        // never swallowed.
        app.confirm_planet();
        assert_eq!(
            app.ship.phase(),
            crate::tui::animation::ShipPhase::Descending
        );

        EventHandler::handle_key(&mut app, key(KeyCode::Enter));

        assert_eq!(
            app.current_view,
            CurrentView::Observatory,
            "Enter inside the dock window must finish the dock and open the observatory"
        );
        assert!(app.is_docked(), "the observatory keeps the dock");
        EventHandler::handle_key(&mut app, key(KeyCode::Enter));
        assert!(!app.is_docked(), "opening the lessons lifts the dock");
        let selected = app.selected_lesson().expect("a lesson must be selected");
        assert_eq!(selected.tier, Tier::Tier2FullAlphabet);
    }

    #[test]
    fn test_planet_click_docked_opens_other_click_selects_and_descends() {
        let area = Rect::new(0, 0, 120, 40);
        let map = map_body_area(area);

        // Arm 1: clicking the DOCKED (selected) card confirms the dock and
        // opens the observatory (D11 click part); its Enter opens the
        // lessons.
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;
        app.ship = crate::tui::animation::ShipAnimation::new(0);
        app.confirm_planet();
        app.finish_ship_animation();
        assert!(app.is_docked());

        let card0 = planet_layout(map, app.selected_planet_index)[0];
        let pos0 = Position::new(card0.x + card0.width / 2, card0.y + card0.height / 2);
        EventHandler::handle_mouse(&mut app, left_click(pos0), area).unwrap();

        assert_eq!(app.current_view, CurrentView::Observatory);
        assert!(app.is_docked(), "the observatory keeps the dock");
        EventHandler::handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.current_view, CurrentView::PlanetLessons);
        let selected = app.selected_lesson().expect("a lesson must be selected");
        assert_eq!(selected.tier, Tier::ALL[0]);

        // Arm 2: clicking ANOTHER card while docked selects it, snaps, and
        // starts a fresh descend — the dock lifts because the selection
        // actually moved (snap_to path).
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;
        app.ship = crate::tui::animation::ShipAnimation::new(0);
        app.confirm_planet();
        app.finish_ship_animation();
        assert!(app.is_docked());

        let card3 = planet_layout(map, app.selected_planet_index)[3];
        let pos3 = Position::new(card3.x + card3.width / 2, card3.y + card3.height / 2);
        EventHandler::handle_mouse(&mut app, left_click(pos3), area).unwrap();

        assert_eq!(app.selected_planet_index, 3);
        assert_eq!(
            app.current_view,
            CurrentView::MainMenu,
            "an other-card click docks at it; it never opens directly"
        );
        assert!(!app.is_docked(), "the snap_to reselect must lift the dock");
        assert_eq!(
            app.ship.phase(),
            crate::tui::animation::ShipPhase::Descending
        );
    }

    #[test]
    fn test_mainmenu_left_right_move_planet_horizontally() {
        // D11 nav remap: Left/h move one planet left, Right/l one planet
        // right, saturating at the band edges. Right no longer opens the
        // lesson list (Enter keeps that job), and a saturating key at an
        // edge starts no ship travel.
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 3;
        app.ship = crate::tui::animation::ShipAnimation::new(3);

        for code in [KeyCode::Left, KeyCode::Char('h')] {
            app.selected_planet_index = 3;
            app.ship = crate::tui::animation::ShipAnimation::new(3);
            EventHandler::handle_key(&mut app, key(code));
            assert_eq!(app.selected_planet_index, 2, "key {code:?} must move left");
            app.ship.complete();
        }
        for code in [KeyCode::Right, KeyCode::Char('l')] {
            app.selected_planet_index = 2;
            app.ship = crate::tui::animation::ShipAnimation::new(2);
            EventHandler::handle_key(&mut app, key(code));
            assert_eq!(app.selected_planet_index, 3, "key {code:?} must move right");
            app.ship.complete();
        }
        assert_eq!(
            app.current_view,
            CurrentView::MainMenu,
            "Right must no longer open the lesson list"
        );

        // Saturating edges: index clamps and the ship stays idle (travel
        // only starts when the selection actually changes).
        app.selected_planet_index = 0;
        app.ship = crate::tui::animation::ShipAnimation::new(0);
        EventHandler::handle_key(&mut app, key(KeyCode::Left));
        assert_eq!(app.selected_planet_index, 0);
        assert!(
            app.ship.is_idle(),
            "Left at the band's left edge must not travel"
        );

        app.selected_planet_index = 6;
        app.ship = crate::tui::animation::ShipAnimation::new(6);
        EventHandler::handle_key(&mut app, key(KeyCode::Right));
        assert_eq!(app.selected_planet_index, 6);
        assert!(
            app.ship.is_idle(),
            "Right at the band's right edge must not travel"
        );
    }

    #[test]
    fn test_mainmenu_up_down_jk_are_noops() {
        // D11: the horizontal band has no vertical planet axis, so
        // Up/Down/j/k are unmapped in MainMenu — no selection change, no
        // travel, no view change.
        for code in [
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::Char('j'),
            KeyCode::Char('k'),
        ] {
            let mut app = App::new();
            app.user_progress = UserProgress::default();
            app.selected_planet_index = 3;
            app.ship = crate::tui::animation::ShipAnimation::new(3);

            EventHandler::handle_key(&mut app, key(code));

            assert_eq!(
                app.selected_planet_index, 3,
                "key {code:?} must not move the selection"
            );
            assert!(app.ship.is_idle(), "key {code:?} must not start a travel");
            assert_eq!(
                app.current_view,
                CurrentView::MainMenu,
                "key {code:?} must not change the view"
            );
        }
    }

    #[test]
    fn test_mainmenu_home_end_preserved() {
        // D11: Home/g and End/G keep their band-jump meaning across the
        // horizontal remap.
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 4;
        app.ship = crate::tui::animation::ShipAnimation::new(4);

        for code in [KeyCode::Home, KeyCode::Char('g')] {
            EventHandler::handle_key(&mut app, key(code));
            assert_eq!(
                app.selected_planet_index, 0,
                "key {code:?} must jump to the band's left end"
            );
            app.ship.complete();
            app.selected_planet_index = 4;
            app.ship = crate::tui::animation::ShipAnimation::new(4);
        }
        for code in [KeyCode::End, KeyCode::Char('G')] {
            EventHandler::handle_key(&mut app, key(code));
            assert_eq!(
                app.selected_planet_index, 6,
                "key {code:?} must jump to the band's right end"
            );
            app.ship.complete();
            app.selected_planet_index = 4;
            app.ship = crate::tui::animation::ShipAnimation::new(4);
        }
    }

    #[test]
    fn test_mainmenu_wheel_scrolls_band() {
        // D11: the wheel scrolls the horizontal band — ScrollUp/ScrollLeft
        // move left, ScrollDown/ScrollRight move right.
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 3;
        app.ship = crate::tui::animation::ShipAnimation::new(3);
        let area = Rect::new(0, 0, 120, 40);

        EventHandler::handle_mouse(
            &mut app,
            mouse(MouseEventKind::ScrollUp, Position::new(10, 10)),
            area,
        )
        .unwrap();
        assert_eq!(app.selected_planet_index, 2, "ScrollUp must move left");
        app.ship.complete();

        EventHandler::handle_mouse(
            &mut app,
            mouse(MouseEventKind::ScrollLeft, Position::new(10, 10)),
            area,
        )
        .unwrap();
        assert_eq!(app.selected_planet_index, 1, "ScrollLeft must move left");
        app.ship.complete();

        EventHandler::handle_mouse(
            &mut app,
            mouse(MouseEventKind::ScrollDown, Position::new(10, 10)),
            area,
        )
        .unwrap();
        assert_eq!(app.selected_planet_index, 2, "ScrollDown must move right");
        app.ship.complete();

        EventHandler::handle_mouse(
            &mut app,
            mouse(MouseEventKind::ScrollRight, Position::new(10, 10)),
            area,
        )
        .unwrap();
        assert_eq!(app.selected_planet_index, 3, "ScrollRight must move right");
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
        assert_eq!(
            app.selected_planet_index,
            Tier::Tier3SpanishOrthography.index()
        );
    }

    #[test]
    fn test_any_keypress_during_animation_completes_it_first() {
        let mut app = App::new();
        app.selected_planet_index = 0;
        app.ship = crate::tui::animation::ShipAnimation::new(0);
        app.move_planet_right();
        // No time has elapsed yet, so without the skip guard the ship would
        // still be interpolating from 0.0 (its `from`) when the next travel
        // retargets it.
        assert_eq!(
            app.ship.phase(),
            crate::tui::animation::ShipPhase::Traveling
        );
        assert!((app.ship.position() - 0.0).abs() < 1e-5);

        EventHandler::handle_key(&mut app, key(KeyCode::Right));

        // The guard must complete the FIRST travel (snapping to planet 1)
        // before the key's own navigation starts a second travel (toward
        // planet 2); the new travel's `from` proves the snap happened,
        // instead of continuing to interpolate from the stale 0.0 origin.
        assert_eq!(
            app.ship.phase(),
            crate::tui::animation::ShipPhase::Traveling
        );
        assert!(
            (app.ship.position() - 1.0).abs() < 1e-5,
            "expected the completed first hop (1.0) as the new travel's origin, got {}",
            app.ship.position()
        );
        assert_eq!(
            app.selected_planet_index, 2,
            "the key itself must still be processed"
        );
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

    #[test]
    fn test_history_esc_enter_q_return_to_mainmenu() {
        for code in [KeyCode::Esc, KeyCode::Enter, KeyCode::Char('q')] {
            let mut app = App::new();
            app.user_progress = UserProgress::default();
            app.current_view = CurrentView::History;

            EventHandler::handle_key(&mut app, key(code));

            assert_eq!(
                app.current_view,
                CurrentView::MainMenu,
                "key {code:?} must return to MainMenu from History"
            );
        }
    }

    /// Regression guard: History's new `b`/`B` binding must not disturb
    /// Stats' existing `e`/`E` binding from MainMenu.
    #[test]
    fn test_stats_binding_unaffected_by_history_addition() {
        for code in [KeyCode::Char('e'), KeyCode::Char('E')] {
            let mut app = App::new();
            app.user_progress = UserProgress::default();

            EventHandler::handle_key(&mut app, key(code));

            assert_eq!(
                app.current_view,
                CurrentView::Stats,
                "key {code:?} must still open Stats from MainMenu"
            );
        }
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
            // D11 horizontal band nav: Left/h and Right/l pan the band;
            // Up/Down/j/k are deliberately unmapped no-ops.
            ("Left", 3, key(KeyCode::Left), MenuEffect::PlanetIndex(2)),
            ("h", 3, key(KeyCode::Char('h')), MenuEffect::PlanetIndex(2)),
            ("Right", 2, key(KeyCode::Right), MenuEffect::PlanetIndex(3)),
            ("l", 2, key(KeyCode::Char('l')), MenuEffect::PlanetIndex(3)),
            ("Up", 3, key(KeyCode::Up), MenuEffect::PlanetIndex(3)),
            ("Down", 3, key(KeyCode::Down), MenuEffect::PlanetIndex(3)),
            ("j", 3, key(KeyCode::Char('j')), MenuEffect::PlanetIndex(3)),
            ("k", 3, key(KeyCode::Char('k')), MenuEffect::PlanetIndex(3)),
            ("End", 0, key(KeyCode::End), MenuEffect::PlanetIndex(6)),
            ("G", 0, key(KeyCode::Char('G')), MenuEffect::PlanetIndex(6)),
            ("Home", 4, key(KeyCode::Home), MenuEffect::PlanetIndex(0)),
            ("g", 4, key(KeyCode::Char('g')), MenuEffect::PlanetIndex(0)),
            (
                "e",
                0,
                key(KeyCode::Char('e')),
                MenuEffect::View(CurrentView::Stats),
            ),
            (
                "v",
                0,
                key(KeyCode::Char('v')),
                MenuEffect::View(CurrentView::Dictation),
            ),
            (
                "b",
                0,
                key(KeyCode::Char('b')),
                MenuEffect::View(CurrentView::History),
            ),
            (
                "B",
                0,
                key(KeyCode::Char('B')),
                MenuEffect::View(CurrentView::History),
            ),
            ("q", 0, key(KeyCode::Char('q')), MenuEffect::Quit),
            (
                "Ctrl+c",
                0,
                KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                MenuEffect::Quit,
            ),
        ];

        for (label, start_index, evt, effect) in cases {
            let mut app = App::new();
            app.user_progress = UserProgress::default();
            app.selected_planet_index = start_index;

            EventHandler::handle_key(&mut app, evt);
            // A uniform advance is safe for every table case — it settles
            // in-flight travels so the assertions land on the snap target.
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
                MenuEffect::Quit => assert!(
                    app.should_quit,
                    "key {label} ({evt:?}) must set should_quit"
                ),
            }
        }

        // Enter is three-step since the observatory flip (dock, confirm
        // into the observatory, then open) and cannot ride the single-key
        // table: first press docks without flipping, second opens the
        // observatory, third opens.
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 2;
        EventHandler::handle_key(&mut app, key(KeyCode::Enter));
        app.advance_animation(crate::tui::animation::DESCEND + std::time::Duration::from_millis(1));
        assert_eq!(
            app.current_view,
            CurrentView::MainMenu,
            "first Enter docks without flipping the view"
        );
        EventHandler::handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(
            app.current_view,
            CurrentView::Observatory,
            "second Enter confirms the dock and opens the observatory"
        );
        EventHandler::handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(
            app.current_view,
            CurrentView::PlanetLessons,
            "third Enter opens the lessons from the observatory"
        );
    }

    #[test]
    fn test_planet_lessons_key_bindings() {
        fn enter_tier1() -> App {
            let mut app = App::new();
            app.user_progress = UserProgress::default();
            app.selected_planet_index = Tier::Tier1Foundation.index();
            // Three-step Enter: reach a genuinely open PlanetLessons view.
            open_selected_planet_lessons(&mut app);
            app
        }

        let tier1_lessons: Vec<_> = App::new()
            .available_lessons()
            .into_iter()
            .filter(|l| l.tier == Tier::Tier1Foundation)
            .collect();
        let first_idx = App::new()
            .flat_index_of(&tier1_lessons.first().unwrap().id)
            .unwrap();
        let last_idx = App::new()
            .flat_index_of(&tier1_lessons.last().unwrap().id)
            .unwrap();

        // Enter starts practice on the currently selected lesson.
        let mut app = enter_tier1();
        EventHandler::handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(
            app.current_view,
            CurrentView::Practice,
            "Enter must start practice"
        );

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
            assert_eq!(
                app.selected_lesson_index, last_idx,
                "key {evt:?} must select the tier's last lesson"
            );
        }

        // Home / g jump to the first lesson of the tier.
        for evt in [key(KeyCode::Home), key(KeyCode::Char('g'))] {
            let mut app = enter_tier1();
            app.selected_lesson_index = last_idx;
            EventHandler::handle_key(&mut app, evt);
            assert_eq!(
                app.selected_lesson_index, first_idx,
                "key {evt:?} must select the tier's first lesson"
            );
        }
    }
}
