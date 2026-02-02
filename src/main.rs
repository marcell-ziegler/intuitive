use crate::{app::App, ui::draw_ui};

mod app;
mod model;
mod ui;

fn main() -> color_eyre::Result<()> {
    let mut term = ratatui::init();
    let mut app = App::default();
    term.draw(|frame| draw_ui(frame, &mut app))?;
    ratatui::restore();
    Ok(())
}
