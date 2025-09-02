pub mod app;
use app::App;
use cli_log::*;

mod numa_node;
mod proc_info;
mod sys_numa_info;
mod ui;

use std::{
    io,
    time::{Duration, Instant},
};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, MouseButton, MouseEventKind},
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
        match event::read()? {
            Event::Key(key) => {
                if key.code == KeyCode::Char('q') {
                    debug!("q pressed");
                    app.exit();
                } else if key.code == KeyCode::Esc {
                    app.hide_popup();
                }
            }
            Event::Mouse(mouse) => {
                if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                    // Store mouse click coordinates for UI processing
                    app.handle_mouse_click(mouse.column, mouse.row);
                }
            }
            _ => {}
        }
    }

    Ok(())
}

pub fn run_app(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    let mut last_tick = Instant::now();

    loop {
        if app.should_exit {
            break;
        }

        app.update();
        terminal.draw(|frame| draw(app, frame))?;
        handle_events(app, last_tick)?;
        last_tick = Instant::now();
    }

    Ok(())
}
