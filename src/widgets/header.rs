use crate::soc::SocInfo;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};

pub fn render(frame: &mut ratatui::Frame, area: Rect, soc: &SocInfo) {
    let block = Block::default()
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
        .title(Line::from(format!(" {} ", soc)).left_aligned())
        .title(
            Line::from(format!(" macy v{} ", env!("CARGO_PKG_VERSION"))).right_aligned(),
        )
        .border_style(Style::default().fg(Color::DarkGray));

    frame.render_widget(block, area);
}
