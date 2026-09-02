use crate::tui::app::{App, CurrentView};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::Rect;
use std::io;

/// Row count moved by `PageUp`/`PageDown`/`Ctrl+b`/`Ctrl+f`/`Ctrl+u`/`Ctrl+d`
/// inside [`CurrentView::PlanetLessons`].
const PAGE_SIZE: usize = 10;

pub struct EventHandler;

impl EventHandler {
    pub fn handle_event(app: &mut App, _area: Rect) -> io::Result<()> {
        if event::poll(app.poll_interval())? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    Self::handle_key(app, key);
                }
                _ => {}
            }
        }
        Ok(())
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
    use crate::core::model::{Tier, UserProgress};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
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
}
