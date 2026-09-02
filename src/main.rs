use crossterm::{
    cursor::Show,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use mecanopro::tui::{render, App, EventHandler};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};
use std::io;
use std::io::stdout;
use std::panic;
use std::time::Instant;

/// Restores the terminal to its normal (non-raw, main-screen, no mouse
/// capture) state. The single implementation shared by both the panic hook
/// and the normal teardown path so they can never drift.
fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture, Show)
}

fn setup_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_panic_hook();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout_handle = stdout();
    execute!(stdout_handle, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout_handle);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    // Main application loop
    while !app.should_quit {
        app.tick(Instant::now());
        terminal.draw(|f| render(f, &app))?;
        let size = terminal.size()?;
        EventHandler::handle_event(&mut app, Rect::new(0, 0, size.width, size.height))?;
    }

    // Restore terminal
    restore_terminal()?;
    terminal.show_cursor()?;

    Ok(())
}
