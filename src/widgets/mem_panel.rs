use super::History;
use crate::sources::memory::MemoryInfo;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Sparkline};

pub fn render(frame: &mut ratatui::Frame, area: Rect, mem: &MemoryInfo, history: &History) {
    let title = format!(
        " Memory  {:.1} / {:.1} GB ",
        mem.used_gb(),
        mem.total_gb()
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Green));

    let sparkline = Sparkline::default()
        .block(block)
        .data(history.data())
        .max(100)
        .style(Style::default().fg(Color::LightGreen));

    frame.render_widget(sparkline, area);
}
