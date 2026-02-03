use color_eyre::eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::{app::App, ui::draw_ui};

mod app;
mod model;
mod ui;

fn main() -> Result<()> {
    let mut term = ratatui::init();
    let mut app = App::default();
    loop {
        term.draw(|frame| draw_ui(frame, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }

            match app.current_screen {
                app::CurrentScreen::Main => match key.code {
                    KeyCode::Esc => break,
                    _ => {}
                },
                _ => {}
            }
        }
    }
    ratatui::restore();
    Ok(())
}
