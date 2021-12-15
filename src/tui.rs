use std::io::{stdout, Stdout};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use tracing::trace;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame, Terminal,
};
use tui_logger::TuiLoggerWidget;
use unicode_width::UnicodeWidthStr;

pub struct Tui {
    terminal: Terminal<Backend>,
    inner: TuiInner,
}
struct TuiInner {
    input_mode: InputMode,
    input: InputField,
}

type Backend = CrosstermBackend<Stdout>;

impl Tui {
    pub fn new() -> eyre::Result<Self> {
        crossterm::terminal::enable_raw_mode()?;
        let backend = CrosstermBackend::new(stdout());
        let mut terminal = Terminal::new(backend)?;

        terminal.clear()?;
        terminal.show_cursor()?;

        Ok(Self {
            terminal,
            inner: TuiInner::new(),
        })
    }

    pub fn tick(&mut self) -> eyre::Result<ControlFlow> {
        self.terminal.draw(|f| self.inner.ui(f))?;

        self.inner.handle_events()
    }

    pub fn cleanup(mut self) -> eyre::Result<()> {
        self.terminal.clear()?;
        crossterm::terminal::disable_raw_mode()?;

        Ok(())
    }
}

impl TuiInner {
    fn new() -> Self {
        Self {
            input_mode: InputMode::Normal,
            input: InputField::new(),
        }
    }

    fn handle_events(&mut self) -> eyre::Result<ControlFlow> {
        if let Event::Key(key) = event::read()? {
            match self.input_mode {
                InputMode::Normal => match key {
                    KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                    } => return Ok(ControlFlow::Halt),
                    KeyEvent {
                        code: KeyCode::Char('/'),
                        ..
                    } => {
                        self.input_mode = InputMode::Input;
                        self.input.begin();
                    }
                    _ => {}
                },
                InputMode::Input => match key {
                    KeyEvent {
                        code: KeyCode::Esc, ..
                    } => self.input_mode = InputMode::Normal,
                    k => self.input.handle_events(k),
                },
                InputMode::Log => match key {
                    KeyEvent {
                        code: KeyCode::Esc, ..
                    } => self.input_mode = InputMode::Normal,
                    _ => {}
                },
            }
        }
        Ok(ControlFlow::Continue)
    }

    fn ui(&mut self, f: &mut Frame<Backend>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
            .split(f.size());
        let log_and_input = Layout::default()
            .direction(Direction::Vertical)
            // .margin(1)
            .constraints([Constraint::Min(10), Constraint::Length(3)].as_ref())
            .split(chunks[1]);

        let input = Spans::from(vec![Span::raw("/"), Span::raw(self.input.current())]);
        let input = Paragraph::new(input).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Input")
                .border_type(BorderType::Rounded),
        );

        f.render_widget(Self::logger(), log_and_input[0]);
        f.render_widget(input, log_and_input[1]);

        match self.input_mode {
            InputMode::Normal => {}
            InputMode::Input => f.set_cursor(
                log_and_input[1].x + self.input.current().width() as u16 + 2,
                log_and_input[1].y + 1,
            ),
            InputMode::Log => {}
        }
    }

    fn logger() -> TuiLoggerWidget<'static> {
        TuiLoggerWidget::default()
            .style_error(Style::default().fg(Color::Red))
            .style_debug(Style::default().fg(Color::Blue))
            .style_warn(Style::default().fg(Color::Yellow))
            .style_trace(Style::default().fg(Color::Magenta))
            .style_info(Style::default().fg(Color::Green))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Logs")
                    .border_type(BorderType::Rounded),
            )
    }
}

struct InputField {
    input: String,
    history: Vec<String>,
    history_preview: Option<usize>,
}

impl InputField {
    fn new() -> Self {
        Self {
            input: String::with_capacity(256),
            history: vec![],
            history_preview: None,
        }
    }
    fn current(&self) -> &str {
        self.history_preview
            .and_then(|ind| self.history.iter().rev().nth(ind))
            .unwrap_or(&self.input)
    }
    fn begin(&mut self) {}
    fn handle_events(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char(ch) => {
                self.history_preview = None;
                self.input.push(ch);
            }
            KeyCode::Backspace => {
                self.history_preview = None;
                self.input.pop();
            }
            KeyCode::Enter => {
                //TODO
                self.history.push(self.current().to_string());
                self.history_preview = None;
                self.input.clear();
            }
            KeyCode::Up => {
                let max_index = self.history.len().checked_sub(1).unwrap_or(0);
                self.history_preview = Some(match self.history_preview {
                    Some(ind) => max_index.min(ind + 1),
                    None => 0,
                });
                trace!(len = self.history.len(), self.history_preview)
            }
            KeyCode::Down => {
                self.history_preview = self.history_preview.and_then(|x| x.checked_sub(1));
                trace!(self.history_preview)
            }
            _ => {}
        }
    }
}

pub enum ControlFlow {
    Halt,
    Continue,
}

enum InputMode {
    Normal,
    Input,
    Log,
}
