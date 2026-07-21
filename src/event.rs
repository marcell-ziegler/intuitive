use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::{
    action::Action,
    app::{App, Panel},
    model::Encounter,
};

pub fn map_event(app: &App, e: &Event) -> Option<Action> {
    if let Some(key_event) = e.as_key_event() {
        match app.current_panel {
            Panel::Editor => handle_editor_keys(&key_event, e),
            Panel::InitiativeTable | Panel::Sidebar => {
                handle_main_view_keys(&app.current_encounter, &key_event, e)
            }
        }
    } else {
        None
    }
}

/// Returns Some(true) if the loop is to be broken
fn handle_main_view_keys(encounter: &Encounter, key_event: &KeyEvent, _: &Event) -> Option<Action> {
    match key_event.code {
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Char('j') | KeyCode::Down => Some(Action::SelectNextRow),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::SelectPreviousRow),
        KeyCode::Char(' ') => Some(Action::AdvanceTurn),
        KeyCode::Tab => Some(Action::SwitchPanel),
        KeyCode::Char('n') => Some(Action::OpenEditorWithNewCreature),
        KeyCode::Delete | KeyCode::Backspace => {
            Some(Action::DeleteCreatureAtIndex(encounter.cursor_index))
        }
        KeyCode::Char('e') => Some(Action::OpenEditorAtIndex(encounter.cursor_index)),
        _ => None,
    }
}

fn handle_editor_keys(key_event: &KeyEvent, e: &Event) -> Option<Action> {
    match key_event.code {
        KeyCode::Char('q') | KeyCode::Esc => Some(Action::CloseEditor),
        KeyCode::Tab | KeyCode::Down => Some(Action::EditorNextField),
        KeyCode::BackTab | KeyCode::Up => Some(Action::EditorPrevField),
        KeyCode::Enter => Some(Action::SubmitEditor),
        KeyCode::Char('t') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::EditorToggleCreatureType)
        }
        _ => Some(Action::EditorInput(e.clone())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::empty()))
    }

    fn ctrl_key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::CONTROL))
    }

    fn editor_app() -> App {
        let mut app = App::default();
        app.current_panel = Panel::Editor;
        app
    }

    #[test]
    fn main_view_maps_navigation_and_lifecycle_keys() {
        let app = App::default(); // defaults to Panel::InitiativeTable
        assert_eq!(
            map_event(&app, &key(KeyCode::Char('j'))),
            Some(Action::SelectNextRow)
        );
        assert_eq!(
            map_event(&app, &key(KeyCode::Down)),
            Some(Action::SelectNextRow)
        );
        assert_eq!(
            map_event(&app, &key(KeyCode::Char('k'))),
            Some(Action::SelectPreviousRow)
        );
        assert_eq!(
            map_event(&app, &key(KeyCode::Up)),
            Some(Action::SelectPreviousRow)
        );
        assert_eq!(
            map_event(&app, &key(KeyCode::Char(' '))),
            Some(Action::AdvanceTurn)
        );
        assert_eq!(
            map_event(&app, &key(KeyCode::Char('n'))),
            Some(Action::OpenEditorWithNewCreature)
        );
        assert_eq!(
            map_event(&app, &key(KeyCode::Char('q'))),
            Some(Action::Quit)
        );
    }

    #[test]
    fn main_view_ignores_unmapped_keys() {
        let app = App::default();
        assert_eq!(map_event(&app, &key(KeyCode::Char('z'))), None);
    }

    #[test]
    fn editor_maps_lifecycle_keys() {
        let app = editor_app();
        assert_eq!(
            map_event(&app, &key(KeyCode::Esc)),
            Some(Action::CloseEditor)
        );
        assert_eq!(
            map_event(&app, &key(KeyCode::Char('q'))),
            Some(Action::CloseEditor)
        );
        assert_eq!(
            map_event(&app, &key(KeyCode::Down)),
            Some(Action::EditorNextField)
        );
        assert_eq!(
            map_event(&app, &key(KeyCode::BackTab)),
            Some(Action::EditorPrevField)
        );
        assert_eq!(
            map_event(&app, &key(KeyCode::Up)),
            Some(Action::EditorPrevField)
        );
        assert_eq!(
            map_event(&app, &key(KeyCode::Enter)),
            Some(Action::SubmitEditor)
        );
    }

    #[test]
    fn editor_falls_through_to_text_input() {
        let app = editor_app();
        let ev = key(KeyCode::Char('x'));
        assert_eq!(map_event(&app, &ev), Some(Action::EditorInput(ev.clone())));
    }

    #[test]
    fn editor_ctrl_t_toggles_creature_type() {
        let app = editor_app();
        assert_eq!(
            map_event(&app, &ctrl_key(KeyCode::Char('t'))),
            Some(Action::EditorToggleCreatureType)
        );
    }

    #[test]
    fn editor_plain_t_is_text_input_not_a_toggle() {
        // Bare `t` must stay ordinary text so names like "Troll" can be typed;
        // only Ctrl+T toggles the creature type.
        let app = editor_app();
        let ev = key(KeyCode::Char('t'));
        assert_eq!(map_event(&app, &ev), Some(Action::EditorInput(ev.clone())));
    }

    #[test]
    fn same_key_maps_differently_per_panel() {
        // Tab switches panels in the table but advances a field in the editor.
        let mut app = App::default();
        assert_eq!(
            map_event(&app, &key(KeyCode::Tab)),
            Some(Action::SwitchPanel)
        );
        app.current_panel = Panel::Editor;
        assert_eq!(
            map_event(&app, &key(KeyCode::Tab)),
            Some(Action::EditorNextField)
        );
    }

    #[test]
    fn non_key_events_map_to_nothing() {
        let app = App::default();
        assert_eq!(map_event(&app, &Event::FocusGained), None);
    }
}
