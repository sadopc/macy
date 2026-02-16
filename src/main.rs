mod app;
mod metrics;
mod soc;
mod sources;
mod widgets;

use clap::Parser;
use std::io;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "macy", about = "macOS Apple Silicon TUI System Monitor")]
struct Args {
    /// Sampling interval in milliseconds
    #[arg(short, long, default_value = "1000")]
    interval: u64,

    /// Print metrics to stdout instead of TUI (for debugging)
    #[arg(long)]
    print: bool,
}

fn main() -> io::Result<()> {
    // Platform check
    if cfg!(not(target_os = "macos")) {
        eprintln!("macy only runs on macOS");
        std::process::exit(1);
    }

    if cfg!(not(target_arch = "aarch64")) {
        eprintln!("macy requires Apple Silicon (arm64)");
        std::process::exit(1);
    }

    let args = Args::parse();
    let interval = Duration::from_millis(args.interval);

    // Detect SoC info
    let soc = soc::detect();

    if args.print {
        println!("SoC: {}", soc);
        println!("GPU freqs: {:?}", soc.gpu_freqs);
        let rx = metrics::start_sampler(interval, soc);
        for _ in 0..3 {
            if let Ok(m) = rx.recv() {
                println!(
                    "CPU: {:.1}% | GPU: {:.1}% @ {:.0}MHz | Mem: {:.1}/{:.1}GB | Power: CPU {:.1}W GPU {:.1}W",
                    m.cpu.overall_percent,
                    m.gpu.utilization, m.gpu.freq_mhz,
                    m.memory.used_gb(), m.memory.total_gb(),
                    m.power.cpu_watts, m.power.gpu_watts,
                );
            }
        }
        return Ok(());
    }

    // Start background sampler
    let rx = metrics::start_sampler(interval, soc.clone());

    // Setup terminal
    ratatui::crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    ratatui::crossterm::execute!(
        stdout,
        ratatui::crossterm::terminal::EnterAlternateScreen,
    )?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    // Run the app
    let mut app = app::App::new(soc, interval);
    let result = app.run(&mut terminal, rx);

    // Restore terminal
    ratatui::crossterm::terminal::disable_raw_mode()?;
    ratatui::crossterm::execute!(
        io::stdout(),
        ratatui::crossterm::terminal::LeaveAlternateScreen,
    )?;
    terminal.show_cursor()?;

    result
}
