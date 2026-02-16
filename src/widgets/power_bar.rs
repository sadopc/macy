use super::History;
use crate::sources::ioreport::PowerMetrics;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Sparkline};

pub fn render(
    frame: &mut ratatui::Frame,
    area: Rect,
    power: &PowerMetrics,
    history: &History,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Power ")
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 2 {
        return;
    }

    // Split inner area: text on top, sparkline on bottom
    let chunks = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(1),
    ])
    .split(inner);

    let total = power.cpu_watts + power.gpu_watts;
    let text = vec![
        Line::from(vec![
            Span::styled("CPU: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{:.1}W", power.cpu_watts)),
            Span::raw("   "),
            Span::styled("GPU: ", Style::default().fg(Color::LightMagenta)),
            Span::raw(format!("{:.1}W", power.gpu_watts)),
        ]),
        Line::from(vec![
            Span::styled("Total: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{:.1}W", total),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, chunks[0]);

    let sparkline = Sparkline::default()
        .data(history.data())
        .style(Style::default().fg(Color::LightYellow));

    frame.render_widget(sparkline, chunks[1]);
}
