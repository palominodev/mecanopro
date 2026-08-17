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
                KeyCode::Up | KeyCode::Char('k') => app.move_selection_up(),
                KeyCode::Down | KeyCode::Char('j') => app.move_selection_down(),
                KeyCode::Enter => {
                    if let Some(lesson) = app.selected_lesson() {
                        app.start_practice(lesson);
                    }
                }
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
                KeyCode::Char('d') | KeyCode::Char('D') => app.start_adaptive_drill(),
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') | KeyCode::Char('Q') => {
                    app.current_view = CurrentView::MainMenu;
                }
                _ => {}
            },
        }
    }
}
