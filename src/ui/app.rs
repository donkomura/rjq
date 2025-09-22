use super::events::{get_action, update};
use crate::app::App;
use crossterm::event::{self, Event, KeyEvent};
use ratatui::{
    Frame, Terminal,
    backend::Backend,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Paragraph, Widget},
};

impl App {
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> crate::Result<()> {
        while !self.should_exit() {
            terminal.draw(|frame| self.draw(frame))?;
            if let Event::Key(key_event) = event::read()? {
                self.handle_events(key_event)?;
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
        frame.set_cursor_position(((self.prompt().len() + self.input().len()) as u16, 0));
    }

    pub fn handle_events(&mut self, key_event: KeyEvent) -> crate::Result<()> {
        let action = get_action(key_event);
        update(self, action);
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(area);

        let prompt_text = format!("{}{}", self.prompt(), self.input());
        let prompt_paragraph = Paragraph::new(prompt_text);
        prompt_paragraph.render(chunks[0], buf);

        if let Some(error) = self.last_error() {
            let error_text = format!("Error: {}", error);
            let error_paragraph = Paragraph::new(error_text);
            error_paragraph.render(chunks[1], buf);
        } else {
            let result_text = match self.execute_current_query() {
                Ok(result) => result.format_pretty(),
                Err(_) => {
                    if self.input().is_empty() {
                        serde_json::to_string_pretty(self.data().get())
                            .unwrap_or_else(|_| "Error formatting JSON".to_string())
                    } else {
                        "".to_string()
                    }
                }
            };

            // Apply scrolling by skipping lines based on scroll_offset
            let lines: Vec<&str> = result_text.lines().collect();
            let available_height = chunks[1].height as usize;

            // Use current scroll offset as-is (bounds are enforced during scroll operations)
            let scroll_offset = self.scroll_offset();
            let visible_lines: Vec<&str> = lines
                .iter()
                .skip(scroll_offset)
                .take(available_height)
                .copied()
                .collect();
            let scrolled_text = visible_lines.join("\n");

            let json_paragraph = Paragraph::new(scrolled_text);
            json_paragraph.render(chunks[1], buf);
        }
    }
}
