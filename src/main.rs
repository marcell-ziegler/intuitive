use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers, ModifierKeyCode};
use tui_input::backend::crossterm::EventHandler;

use crate::{
    app::{EditorField, Panel},
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
            match app.current_panel {
                Panel::Editor => {
                    // Custom keybinds for editor window
                    // Handle event in appropriate input else
                    if let Some(key_event) = e.as_key_event() {
                        match key_event.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                app.current_panel = Panel::InitiativeTable;
                                // TODO: Clear input states
                            }
                            KeyCode::Tab | KeyCode::Down => {
                                app.editor_state.next_field();
                            }
                            KeyCode::BackTab | KeyCode::Up => {
                                app.editor_state.previous_fied();
                            }
                            KeyCode::Enter => {
                                app.submit_editor();
                            }
                        }
                    } else {
                        match app.editor_state.active_input {
                            EditorField::Name => {
                                app.editor_state.name_input.handle_event(&e);
                            }
                            _ => continue,
                        }
                    }
                }
                _ => {
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
                            KeyCode::Tab => {
                                app.current_panel = match app.current_panel {
                                    Panel::InitiativeTable => Panel::Sidebar,
                                    Panel::Sidebar => Panel::InitiativeTable,
                                    _ => app.current_panel,
                                }
                            }
                            _ => continue,
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

fn handle_main_view_key_event(event: &KeyEvent) {}
