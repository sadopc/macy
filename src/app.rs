use crate::metrics::Metrics;
use crate::soc::SocInfo;
use crate::widgets::{cpu_panel, gpu_panel, header, mem_panel, power_bar, History};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};
use std::io;
use std::sync::mpsc;
use std::time::Duration;

const HISTORY_CAP: usize = 120;

pub struct App {
    soc: SocInfo,
    current: Metrics,
    cpu_history: History,
    gpu_history: History,
    mem_history: History,
    power_history: History,
    interval: Duration,
}

impl App {
    pub fn new(soc: SocInfo, interval: Duration) -> Self {
        Self {
            soc,
            current: Metrics::default(),
            cpu_history: History::new(HISTORY_CAP),
            gpu_history: History::new(HISTORY_CAP),
            mem_history: History::new(HISTORY_CAP),
            power_history: History::new(HISTORY_CAP),
            interval,
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<impl ratatui::backend::Backend>,
        rx: mpsc::Receiver<Metrics>,
    ) -> io::Result<()> {
        loop {
            // Poll for keyboard events
            if ratatui::crossterm::event::poll(Duration::from_millis(100))? {
                if let ratatui::crossterm::event::Event::Key(key) =
                    ratatui::crossterm::event::read()?
                {
                    use ratatui::crossterm::event::KeyCode;
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                        KeyCode::Esc => return Ok(()),
                        _ => {}
                    }
                    // Also handle Ctrl+C
                    if key.modifiers.contains(ratatui::crossterm::event::KeyModifiers::CONTROL)
                        && key.code == ratatui::crossterm::event::KeyCode::Char('c')
                    {
                        return Ok(());
                    }
                }
            }

            // Check for new metrics (non-blocking)
            while let Ok(metrics) = rx.try_recv() {
                self.cpu_history.push(metrics.cpu.overall_percent as u64);
                self.gpu_history.push(metrics.gpu.utilization as u64);
                self.mem_history.push(metrics.memory.usage_percent() as u64);
                let total_power = (metrics.power.cpu_watts + metrics.power.gpu_watts) * 10.0;
                self.power_history.push(total_power as u64); // Store in 0.1W units
                self.current = metrics;
            }

            // Render
            terminal.draw(|frame| self.render(frame))?;
        }
    }

    fn render(&self, frame: &mut ratatui::Frame) {
        let area = frame.area();

        // Outer layout: header, body, footer
        let outer = Layout::vertical([
            Constraint::Length(1), // header
            Constraint::Min(6),   // body
            Constraint::Length(1), // footer
        ])
        .split(area);

        // Header
        header::render(frame, outer[0], &self.soc);

        // Body: two rows
        let body = Layout::vertical([
            Constraint::Fill(1), // CPU + GPU row
            Constraint::Fill(1), // Memory + Power row
        ])
        .split(outer[1]);

        // First row: CPU | GPU
        let row1 = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .split(body[0]);

        cpu_panel::render(frame, row1[0], self.current.cpu.overall_percent, &self.cpu_history);
        gpu_panel::render(frame, row1[1], &self.current.gpu, &self.gpu_history);

        // Second row: Memory | Power
        let row2 = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .split(body[1]);

        mem_panel::render(frame, row2[0], &self.current.memory, &self.mem_history);
        power_bar::render(frame, row2[1], &self.current.power, &self.power_history);

        // Footer
        let interval_ms = self.interval.as_millis();
        let interval_str = if interval_ms >= 1000 {
            format!("{}s", interval_ms / 1000)
        } else {
            format!("{}ms", interval_ms)
        };

        let footer = Block::default()
            .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
            .title(Line::from(" q: quit ").left_aligned())
            .title(Line::from(format!(" interval {} ", interval_str)).right_aligned())
            .border_style(Style::default().fg(Color::DarkGray));

        frame.render_widget(footer, outer[2]);
    }
}
