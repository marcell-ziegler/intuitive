use crossterm::event::{self, KeyCode};

use crate::ui::draw_ui;

mod app;
mod model;
mod storage;
mod ui;

fn main() -> color_eyre::Result<()> {
    debug_assert!(dotenvy::dotenv().is_ok());

    let mut term = ratatui::init();
    let mut app = storage::load_state()?.unwrap_or_default();
    app.sync_table_state();

    loop {
        term.draw(|frame| draw_ui(frame, &mut app))?;
        if let Ok(e) = event::read() {
            if let Some(key_event) = e.as_key_event() {
                match key_event.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') | KeyCode::Down => {
                        app.select_next_row();
                        storage::store_state(&app)?;
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        app.select_previous_row();
                        storage::store_state(&app)?;
                    }
                    KeyCode::Char(' ') => {
                        app.increment_initiative_order();
                        storage::store_state(&app)?;
                    }
                    _ => continue,
                }
            }
        }
    }
    storage::store_state(&app)?;
    ratatui::restore();
    Ok(())
}
