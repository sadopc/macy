use super::History;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Sparkline};

pub fn render(frame: &mut ratatui::Frame, area: Rect, usage: f64, history: &History) {
    let title = format!(" CPU  {:.0}% ", usage);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Blue));

    let sparkline = Sparkline::default()
        .block(block)
        .data(history.data())
        .max(100)
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(sparkline, area);
}
