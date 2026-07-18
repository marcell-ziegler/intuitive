use crossterm::event::{Event, KeyCode, KeyEvent};

use crate::{
    action::Action,
    app::{App, Panel},
};

pub fn map_event(app: &App, e: &Event) -> Option<Action> {
    if let Some(key_event) = e.as_key_event() {
        match app.current_panel {
            Panel::Editor => handle_editor_keys(&key_event, &e),
            Panel::InitiativeTable | Panel::Sidebar => handle_main_view_keys(&key_event, &e),
        }
    } else {
        None
    }
}

/// Returns Some(true) if the loop is to be broken
fn handle_main_view_keys(key_event: &KeyEvent, _: &Event) -> Option<Action> {
    match key_event.code {
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Char('j') | KeyCode::Down => Some(Action::SelectNextRow),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::SelectPreviousRow),
        KeyCode::Char(' ') => Some(Action::AdvanceTurn),
        KeyCode::Tab => Some(Action::SwitchPanel),
        KeyCode::Char('n') => Some(Action::OpenEditorWithNewCreature),
        _ => None,
    }
}

fn handle_editor_keys(key_event: &KeyEvent, e: &Event) -> Option<Action> {
    match key_event.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            // app.current_panel = Panel::InitiativeTable;
            // TODO: Clear input states
            Some(Action::CloseEditor)
        }
        KeyCode::Tab | KeyCode::Down => {
            // app.editor_state.next_field();
            Some(Action::EditorNextField)
        }
        KeyCode::BackTab | KeyCode::Up => {
            // app.editor_state.previous_field();
            Some(Action::EditorPrevField)
        }
        KeyCode::Enter => {
            // app.submit_editor();
            Some(Action::SubmitEditor)
        }
        _ => Some(Action::EditorInput(e.clone())),
    }
}
