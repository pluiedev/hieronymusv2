use std::{
    io::{stdout, Stdout},
    time::Duration,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use tui::{
    backend::CrosstermBackend,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
    Terminal,
};
use tui_logger::TuiLoggerWidget;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    tick_rate: Duration,
}

impl Tui {
    pub fn new() -> eyre::Result<Self> {
        crossterm::terminal::enable_raw_mode()?;
        let backend = CrosstermBackend::new(stdout());
        let mut terminal = Terminal::new(backend)?;
        let tick_rate = Duration::from_millis(200);

        terminal.clear()?;
        terminal.hide_cursor()?;

        Ok(Self {
            terminal,
            tick_rate,
        })
    }

    pub fn tick(&mut self) -> eyre::Result<ControlFlow> {
        self.terminal.draw(|f| {
            let size = f.size();
            f.render_widget(Self::logger(), size);
        })?;

        // poll for tick rate duration, if no event, sent tick event.
        if event::poll(self.tick_rate)? {
            if let Ok(Event::Key(KeyEvent { code, modifiers })) = event::read() {
                match (code, modifiers) {
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(ControlFlow::Halt),
                    _ => {}
                }
            }
        }
        Ok(ControlFlow::Continue)
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
                    .border_type(BorderType::Plain),
            )
    }

    pub fn cleanup(mut self) -> eyre::Result<()> {
        self.terminal.clear()?;
        self.terminal.show_cursor()?;
        crossterm::terminal::disable_raw_mode()?;

        Ok(())
    }
}

pub enum ControlFlow {
    Halt,
    Continue,
}
