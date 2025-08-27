use cli_log::*;
use numatop::{app::App, run_app};
use ratatui::crossterm::{
    ExecutableCommand,
    event::{DisableMouseCapture, EnableMouseCapture},
};

use std::io;

fn main() -> io::Result<()> {
    init_cli_log!();
    io::stdout().execute(EnableMouseCapture)?;
    let mut terminal = ratatui::init();
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);
    ratatui::restore();
    io::stdout().execute(DisableMouseCapture)?;
    res
}
