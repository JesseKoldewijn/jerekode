//! Optional native TUI path behind the `native-tui` feature.

#[cfg(feature = "native-tui")]
mod imp {
    use ratatui::backend::TestBackend;
    use ratatui::widgets::{Block, Borders, Paragraph};
    use ratatui::Terminal;

    /// Render a minimal status frame (test-backend; no real tty required).
    pub fn render_stub_frame(title: &str) -> String {
        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                let area = frame.area();
                let block = Block::default().title(title).borders(Borders::ALL);
                let paragraph = Paragraph::new("jereko native-tui stub").block(block);
                frame.render_widget(paragraph, area);
            })
            .expect("draw");
        format!("native-tui:{title}")
    }
}

#[cfg(feature = "native-tui")]
pub use imp::render_stub_frame;

#[cfg(not(feature = "native-tui"))]
pub fn render_stub_frame(_title: &str) -> String {
    "native-tui:disabled".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_frame_compiles() {
        let out = render_stub_frame("jereko");
        assert!(!out.is_empty());
    }
}
