use crossterm::{
    execute,
    terminal::{LeaveAlternateScreen, disable_raw_mode},
};
use ratatui::{Terminal, backend::Backend};

pub fn restore_terminal<B: Backend + std::io::Write>(terminal: &mut Terminal<B>) -> std::result::Result<(), std::io::Error> {
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    disable_raw_mode()?;
    terminal.show_cursor()?;
    Ok(())
}