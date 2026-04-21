use crossterm::event::{self, KeyCode};

use crate::{app::App, model::Creature, ui::draw_ui};

mod app;
mod model;
mod storage;
mod ui;

fn main() -> color_eyre::Result<()> {
    debug_assert!(dotenvy::dotenv().is_ok());

    let mut term = ratatui::init();
    let mut app = App::default();

    app.add_creature(Creature::new_player(
        "John Doe",
        32,
        12,
        Some(15),
        None,
        Some(2),
    ));
    app.add_creature(Creature::new_player(
        "Bertil Jansson",
        24,
        12,
        None,
        None,
        Some(3),
    ));

    loop {
        term.draw(|frame| draw_ui(frame, &mut app))?;
        if let Ok(e) = event::read() {
            if let Some(key_event) = e.as_key_event() {
                match key_event.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') | KeyCode::Down => app.select_next_row(),
                    KeyCode::Char('k') | KeyCode::Up => app.select_previous_row(),
                    KeyCode::Char(' ') => app.increment_initiative_order(),
                    _ => continue,
                }
            }
        }
    }
    ratatui::restore();
    Ok(())
}
