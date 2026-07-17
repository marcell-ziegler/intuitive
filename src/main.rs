use crossterm::event::{self, Event, KeyCode, KeyEvent};

use crate::{action::Action, app::Panel, ui::draw_ui};

mod action;
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
                // Capture current action
                let action = match app.current_panel {
                    Panel::Editor => handle_editor_keys(&key_event, &e)?,
                    Panel::InitiativeTable | Panel::Sidebar => {
                        handle_main_view_keys(&key_event, &e)?
                    }
                };

                if let Some(a) = action {
                    app.update(a);
                }
            }
        }
    }
    storage::store_state(&app)?;
    ratatui::restore();
    Ok(())
}

/// Returns Some(true) if the loop is to be broken
fn handle_main_view_keys(key_event: &KeyEvent, _: &Event) -> color_eyre::Result<Option<Action>> {
    match key_event.code {
        KeyCode::Char('q') => Ok(Some(Action::Quit)),
        KeyCode::Char('j') | KeyCode::Down => Ok(Some(Action::SelectNextRow)),
        KeyCode::Char('k') | KeyCode::Up => Ok(Some(Action::SelectPreviousRow)),
        KeyCode::Char(' ') => Ok(Some(Action::AdvanceTurn)),
        KeyCode::Tab => Ok(Some(Action::SwitchPanel)),
        KeyCode::Char('n') => Ok(Some(Action::OpenEditor)),
        _ => Ok(None),
    }
}

fn handle_editor_keys(key_event: &KeyEvent, e: &Event) -> color_eyre::Result<Option<Action>> {
    match key_event.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            // app.current_panel = Panel::InitiativeTable;
            // TODO: Clear input states
            Ok(Some(Action::CloseEditor))
        }
        KeyCode::Tab | KeyCode::Down => {
            // app.editor_state.next_field();
            Ok(Some(Action::EditorNextField))
        }
        KeyCode::BackTab | KeyCode::Up => {
            // app.editor_state.previous_field();
            Ok(Some(Action::EditorPrevField))
        }
        KeyCode::Enter => {
            // app.submit_editor();
            Ok(Some(Action::SubmitEditor))
        }
        _ => Ok(Some(Action::EditorInput(e.clone()))),
    }
}
