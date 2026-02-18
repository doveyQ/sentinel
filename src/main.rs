use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use sysinfo::{Disks, Networks, System};

mod stats;
mod ui;

use stats::SystemStats;

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();

    let result = run(&mut terminal);

    ratatui::restore();
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
    let mut sys = System::new_all();
    let mut disks = Disks::new_with_refreshed_list();
    let mut networks = Networks::new_with_refreshed_list();

    let refresh_interval = Duration::from_secs(2);
    let mut last_refresh = Instant::now() - refresh_interval; // force immediate first draw
    let mut stats = SystemStats::collect(&mut sys, &mut disks, &mut networks);

    loop {
        // Refresh stats on interval
        if last_refresh.elapsed() >= refresh_interval {
            stats = SystemStats::collect(&mut sys, &mut disks, &mut networks);
            last_refresh = Instant::now();
        }

        // Draw
        terminal.draw(|frame| ui::draw(frame, &stats))?;

        // Non-blocking event poll (200ms to stay responsive)
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                        KeyCode::Esc => return Ok(()),
                        _ => {}
                    }
                }
            }
        }
    }
}