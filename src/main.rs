use crossterm::event::{Event, KeyCode, KeyEvent};

use crate::{
    action::Action,
    app::{Effect, Panel},
    event::map_event,
    ui::draw_ui,
};

mod action;
mod app;
mod event;
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
        if let Ok(e) = crossterm::event::read() {
            // Capture current action
            let action = map_event(&app, &e);
            if let Some(a) = action {
                if app.update(a) == Effect::Quit {
                    break;
                }
            }
        }
    }
    storage::store_state(&app)?;
    ratatui::restore();
    Ok(())
}
