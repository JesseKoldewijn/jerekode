//! Optional native TUI path behind the `native-tui` feature.

#[cfg(feature = "native-tui")]
#[allow(clippy::collapsible_if)]
mod imp {
    use crossterm::ExecutableCommand;
    use crossterm::event::{self, Event, KeyCode, KeyEventKind};
    use crossterm::terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    };
    use ratatui::Terminal;
    use ratatui::backend::{CrosstermBackend, TestBackend};
    use ratatui::widgets::{Block, Borders, Paragraph};
    use std::io::{self, stdout};
    use std::time::Duration;

    /// Render a minimal status frame (test-backend; no real tty required).
    pub fn render_stub_frame(title: &str) -> String {
        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                let area = frame.area();
                let block = Block::default().title(title).borders(Borders::ALL);
                let paragraph = Paragraph::new("jerekode native-tui stub").block(block);
                frame.render_widget(paragraph, area);
            })
            .expect("draw");
        format!("native-tui:{title}")
    }

    /// Interactive MVP loop: draw status, quit on `q` / Esc.
    pub fn run_interactive(title: &str) -> io::Result<String> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout());
        let mut terminal = Terminal::new(backend)?;
        let mut status = format!("{title} — press q to quit");

        loop {
            terminal.draw(|frame| {
                let area = frame.area();
                let block = Block::default()
                    .title("jerekode native-tui")
                    .borders(Borders::ALL);
                let paragraph = Paragraph::new(status.as_str()).block(block);
                frame.render_widget(paragraph, area);
            })?;

            if event::poll(Duration::from_millis(200))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Char(c) => status = format!("key:{c}"),
                            _ => {}
                        }
                    }
                }
            }
        }

        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok("native-tui:exited".into())
    }
}

#[cfg(feature = "native-tui")]
pub use imp::{render_stub_frame, run_interactive};

#[cfg(not(feature = "native-tui"))]
pub fn render_stub_frame(_title: &str) -> String {
    "native-tui:disabled".into()
}

#[cfg(not(feature = "native-tui"))]
pub fn run_interactive(_title: &str) -> std::io::Result<String> {
    Ok("native-tui:disabled".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_frame_compiles() {
        let out = render_stub_frame("jerekode");
        assert!(!out.is_empty());
    }
}
