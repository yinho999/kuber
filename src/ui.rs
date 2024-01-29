use std::io;
use std::io::Stdout;
use tui::backend::{Backend, CrosstermBackend};
use crate::App;
use tui::Terminal;
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    mut app: App,
) -> Result<(), io::Error> {
    Ok(())
}