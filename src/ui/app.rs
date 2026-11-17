use super::events::{get_action, update};
use super::syntax::SyntaxHighlighter;
use crate::app::{App, LoadState};
use crate::input::InputEvent;
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
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

const INPUT_BUDGET_PER_FRAME: Duration = Duration::from_millis(16);
// While loading, waking up every frame lets newly parsed values show up promptly
// without spinning when the input arrives slowly (e.g. `tail -f log.jsonl | rjq`).
const LOADING_POLL_INTERVAL: Duration = Duration::from_millis(16);
const IDLE_POLL_INTERVAL: Duration = Duration::from_millis(250);

impl App {
    pub fn run<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        inputs: Receiver<InputEvent>,
    ) -> crate::Result<()> {
        while !self.should_exit() {
            self.receive_inputs(&inputs, Instant::now() + INPUT_BUDGET_PER_FRAME);
            self.tick(Instant::now());
            terminal.draw(|frame| self.draw(frame))?;
            let interval = match self.load_state() {
                LoadState::Loading => LOADING_POLL_INTERVAL,
                _ => IDLE_POLL_INTERVAL,
            };
            let wait = match self.next_deadline() {
                Some(deadline) => interval.min(deadline.saturating_duration_since(Instant::now())),
                None => interval,
            };
            if event::poll(wait)?
                && let Event::Key(key_event) = event::read()?
            {
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

        if let Some(candidate) = suggestion.as_ref().and_then(|c| c.strip_prefix(input)) {
            // 入力済み部分 + 候補部分の表示
            // 通常色で入力部分
            let input_text = format!("{}{}", prompt, input);
            let input_span = Span::styled(input_text, Style::default());

            // グレー色で候補部分
            let suggestion_span = Span::styled(candidate, Style::default().fg(Color::DarkGray));

            let line = Line::from(vec![input_span, suggestion_span]);
            let paragraph = Paragraph::new(line);
            paragraph.render(area, buf);
            return;
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
            .constraints([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);

        // プロンプト行を候補付きで描画
        self.render_input_with_suggestion(chunks[0], buf);

        if let Some(error) = self.last_error() {
            let error_text = format!("Error: {}", error);
            let error_paragraph = Paragraph::new(error_text);
            error_paragraph.render(chunks[1], buf);
        } else {
            let visible_lines = self.visible_lines(chunks[1].height as usize);

            // JSONにシンタックスハイライトを適用
            let highlighter = SyntaxHighlighter::with_theme(*self.theme());
            let highlighted_lines: Vec<Line> = visible_lines
                .iter()
                .map(|line| highlighter.highlight_line(line))
                .collect();

            let json_paragraph = Paragraph::new(highlighted_lines);
            json_paragraph.render(chunks[1], buf);
        }

        Paragraph::new(self.status_line())
            .style(Style::default().fg(Color::DarkGray))
            .render(chunks[2], buf);
    }
}
