use std::time::{Duration, Instant};

use crate::{app::Effect, event::map_event, ui::draw_ui};

mod action;
mod app;
mod editor;
mod event;
mod model;
mod storage;
mod ui;

/// How long a mutation may sit as `state_dirty` before the main loop flushes it
/// to disk on its own. Also the poll timeout, so the loop wakes up to check the
/// dirty timer even when idle (no input).
const AUTOSAVE_INTERVAL: Duration = Duration::from_secs(3);

fn main() -> color_eyre::Result<()> {
    debug_assert!(dotenvy::dotenv().is_ok());

    let mut term = ratatui::init();
    let mut app = storage::load_state()?.unwrap_or_default();

    let mut last_save = Instant::now();

    loop {
        term.draw(|frame| draw_ui(frame, &mut app))?;

        if crossterm::event::poll(AUTOSAVE_INTERVAL)? {
            let e = crossterm::event::read()?;
            let action = map_event(&app, &e);
            if let Some(a) = action {
                match app.update(a) {
                    Effect::Quit => break,
                    Effect::UpdateState => {
                        storage::store_state(&app)?;
                        app.state_dirty = false;
                        last_save = Instant::now();
                    }
                    Effect::None => {}
                }
            }
        }

        if app.state_dirty && last_save.elapsed() >= AUTOSAVE_INTERVAL {
            storage::store_state(&app)?;
            app.state_dirty = false;
            last_save = Instant::now();
        }
    }

    // Always persist at the end
    storage::store_state(&app)?;
    ratatui::restore();
    Ok(())
}
