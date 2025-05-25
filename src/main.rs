use numatop::{app::App, run_app};

use std::io;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);
    ratatui::restore();
    res
}
