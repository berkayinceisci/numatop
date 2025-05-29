pub mod app;
use app::App;

mod numa_node;
mod proc_cpu_info;
mod sys_numa_info;
mod ui;

use std::{
    io,
    time::{Duration, Instant},
};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode},
};

const TICK_RATE: Duration = Duration::from_millis(1000); // Refresh every 1 second

fn draw(app: &mut App, frame: &mut Frame) {
    ui::draw(app, frame);
}

fn handle_events(app: &mut App, last_tick: Instant) -> io::Result<()> {
    let timeout = TICK_RATE
        .checked_sub(last_tick.elapsed())
        .unwrap_or_else(|| Duration::from_secs(0));

    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                todo!("exit");
            }
        }
    }

    Ok(())
}

pub fn run_app(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    let mut last_tick = Instant::now();

    loop {
        app.update();
        terminal.draw(|frame| draw(app, frame))?;
        handle_events(app, last_tick)?;
        last_tick = Instant::now();
    }
}
