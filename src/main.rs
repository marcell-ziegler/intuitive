use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, ModifierKeyCode};
use tui_input::backend::crossterm::EventHandler;

use crate::{
    app::{App, EditorField, Panel},
    ui::draw_ui,
};

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
                match app.current_panel {
                    Panel::Editor => handle_editor_events(&mut app, &key_event, &e),
                    Panel::InitiativeTable | Panel::Sidebar => {
                        if handle_panel_and_sidebar_events(&mut app, &key_event, &e)? {
                            break;
                        }
                    }
                }
            }
        }
    }
    storage::store_state(&app)?;
    ratatui::restore();
    Ok(())
}

/// Returns if the loop is to be broken
fn handle_panel_and_sidebar_events(
    app: &mut App,
    key_event: &KeyEvent,
    e: &Event,
) -> color_eyre::Result<bool> {
    match key_event.code {
        KeyCode::Char('q') => return Ok(true),
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
        KeyCode::Tab => {
            app.current_panel = match app.current_panel {
                Panel::InitiativeTable => Panel::Sidebar,
                Panel::Sidebar => Panel::InitiativeTable,
                _ => app.current_panel,
            }
        }
        KeyCode::Char('n') => app.current_panel = Panel::Editor,
        _ => return Ok(false),
    };
    Ok(false)
}

fn handle_editor_events(app: &mut App, key_event: &KeyEvent, e: &Event) {
    match key_event.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.current_panel = Panel::InitiativeTable;
            // TODO: Clear input states
        }
        KeyCode::Tab | KeyCode::Down => {
            app.editor_state.next_field();
        }
        KeyCode::BackTab | KeyCode::Up => {
            app.editor_state.previous_field();
        }
        KeyCode::Enter => {
            app.submit_editor();
        }
        _ => handle_editor_input_event_delegation(app, &e),
    }
}

fn handle_editor_input_event_delegation(app: &mut App, e: &Event) {
    match app.editor_state.active_input {
        EditorField::Name => {
            app.editor_state.name_input.handle_event(&e);
        }
        EditorField::CurrentHP => {
            app.editor_state.cur_hp_input.handle_event(&e);
        }
        EditorField::MaxHP => {
            app.editor_state.max_hp_input.handle_event(&e);
        }
        EditorField::AC => {
            app.editor_state.ac_input.handle_event(&e);
        }
        EditorField::CR => {
            app.editor_state.cr_input.handle_event(&e);
        }
        _ => (),
    }
}
fn handle_main_view_key_event(event: &KeyEvent) {}
