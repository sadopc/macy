use super::History;
use crate::sources::ioreport::GpuMetrics;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Sparkline};

pub fn render(frame: &mut ratatui::Frame, area: Rect, gpu: &GpuMetrics, history: &History) {
    let title = if gpu.freq_mhz > 0.0 {
        format!(" GPU  {:.0}% @ {:.0} MHz ", gpu.utilization, gpu.freq_mhz)
    } else {
        format!(" GPU  {:.0}% ", gpu.utilization)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Magenta));

    let sparkline = Sparkline::default()
        .block(block)
        .data(history.data())
        .max(100)
        .style(Style::default().fg(Color::LightMagenta));

    frame.render_widget(sparkline, area);
}
