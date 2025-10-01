use super::events::{get_action, update};
use super::syntax::SyntaxHighlighter;
use crate::app::App;
use crossterm::event::{self, Event, KeyEvent};
use ratatui::{
    Frame, Terminal,
    backend::Backend,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
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

    fn render_input_with_suggestion(&self, area: Rect, buf: &mut Buffer) {
        let prompt = self.prompt();
        let input = self.input();

        // 最適候補を取得
        let suggestion = self.get_best_suggestion();

        if let Some(candidate) = suggestion {
            if let Some(completed_part) = candidate.strip_prefix(input) {
                // 入力済み部分 + 候補部分の表示
                // 通常色で入力部分
                let input_text = format!("{}{}", prompt, input);
                let input_span = Span::styled(input_text, Style::default());

                // グレー色で候補部分
                let suggestion_span =
                    Span::styled(completed_part, Style::default().fg(Color::DarkGray));

                let line = Line::from(vec![input_span, suggestion_span]);
                let paragraph = Paragraph::new(line);
                paragraph.render(area, buf);
                return;
            }
        }

        // 候補がない場合は通常表示
        let prompt_text = format!("{}{}", prompt, input);
        let paragraph = Paragraph::new(prompt_text);
        paragraph.render(area, buf);
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(area);

        // プロンプト行を候補付きで描画
        self.render_input_with_suggestion(chunks[0], buf);

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

            // JSONにシンタックスハイライトを適用
            let highlighter = SyntaxHighlighter::new();
            let highlighted_lines: Vec<Line> = visible_lines
                .iter()
                .map(|line| highlighter.highlight_line(line))
                .collect();

            let json_paragraph = Paragraph::new(highlighted_lines);
            json_paragraph.render(chunks[1], buf);
        }
    }
}
