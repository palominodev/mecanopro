use crate::tui::app::{App, CurrentView};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::io;
use std::time::Duration;

pub struct EventHandler;

impl EventHandler {
    pub fn handle_event(app: &mut App) -> io::Result<()> {
        if event::poll(Duration::from_millis(50))? {
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

        match app.current_view {
            CurrentView::MainMenu => match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
                KeyCode::Up | KeyCode::Char('k') => app.move_planet_up(),
                KeyCode::Down | KeyCode::Char('j') => app.move_planet_down(),
                KeyCode::Home | KeyCode::Char('g') => app.planet_home(),
                KeyCode::End | KeyCode::Char('G') => app.planet_end(),
                KeyCode::Enter => {} // wired in PR4/5: enter_planet_lessons()
                KeyCode::Char('v') | KeyCode::Char('V') => app.start_dictation(None, None),
                KeyCode::Char('d') | KeyCode::Char('D') => app.start_adaptive_drill(),
                KeyCode::Char('e') | KeyCode::Char('E') => app.current_view = CurrentView::Stats,
                _ => {}
            },

            CurrentView::Practice => match key.code {
                KeyCode::Esc => app.current_view = CurrentView::MainMenu,
                KeyCode::Tab => app.restart_current_session(),
                KeyCode::Char(ch) => app.handle_key_input(ch),
                _ => {}
            },

            CurrentView::Summary => match key.code {
                KeyCode::Char('r') | KeyCode::Char('R') => app.restart_current_session(),
                KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Enter => app.next_lesson(),
                KeyCode::Esc | KeyCode::Char('m') | KeyCode::Char('M') => app.current_view = CurrentView::MainMenu,
                _ => {}
            },

            CurrentView::Stats => match key.code {
                KeyCode::Char('v') | KeyCode::Char('V') => app.start_dictation(None, None),
                KeyCode::Char('d') | KeyCode::Char('D') => app.start_adaptive_drill(),
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') | KeyCode::Char('Q') => {
                    app.current_view = CurrentView::MainMenu;
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
                        app.current_view = CurrentView::MainMenu;
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
                KeyCode::Esc | KeyCode::Char('m') | KeyCode::Char('M') => app.current_view = CurrentView::MainMenu,
                _ => {}
            },

            // wired in PR5: planet lessons list navigation and drill-down
            CurrentView::PlanetLessons => {}
        }
    }
}
