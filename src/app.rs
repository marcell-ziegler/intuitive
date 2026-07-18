use crossterm::event::Event;
use ratatui::widgets::TableState;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

use crate::{action::Action, model::Creature, storage::Encounter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Panel {
    InitiativeTable,
    Sidebar,
    Editor,
}

#[derive(Debug, Clone)]
pub struct App {
    pub main_table_state: TableState,
    pub current_encounter: Encounter,
    pub current_panel: Panel,
    pub editor_state: EditorState,
    pub dirty: bool,
}

#[derive(Clone, Debug, Default)]
pub struct EditorState {
    pub name_input: Input,
    pub max_hp_input: Input,
    pub cur_hp_input: Input,
    pub ac_input: Input,
    pub cr_input: Input,
    pub amount_input: Input,
    pub active_input: EditorField,
}

impl EditorState {
    pub fn next_field(&mut self) {
        self.active_input = match self.active_input {
            EditorField::Name => EditorField::CurrentHP,
            EditorField::CurrentHP => EditorField::MaxHP,
            EditorField::MaxHP => EditorField::AC,
            EditorField::AC => EditorField::CR,
            EditorField::CR => EditorField::Amount,
            EditorField::Amount | EditorField::Unfocused => EditorField::Name,
        };
    }
    pub fn previous_field(&mut self) {
        self.active_input = match self.active_input {
            EditorField::Name => EditorField::Amount,
            EditorField::CurrentHP => EditorField::Name,
            EditorField::MaxHP => EditorField::CurrentHP,
            EditorField::AC => EditorField::MaxHP,
            EditorField::CR => EditorField::AC,
            EditorField::Amount | EditorField::Unfocused => EditorField::CR,
        };
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
pub enum EditorField {
    #[default]
    Name,
    MaxHP,
    CurrentHP,
    AC,
    CR,
    Amount,
    Unfocused,
}

pub struct NumInput {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableApp {
    current_encounter: Encounter,
    current_panel: Panel,
}

impl From<&App> for SerializableApp {
    fn from(app: &App) -> Self {
        Self {
            current_encounter: app.current_encounter.clone(),
            current_panel: app.current_panel,
        }
    }
}

impl From<SerializableApp> for App {
    fn from(value: SerializableApp) -> Self {
        let mut main_table_state = TableState::default();
        if value.current_encounter.creatures.is_empty() {
            main_table_state.select(None);
        } else {
            main_table_state.select(Some(value.current_encounter.initiative_index));
        }

        let mut app = Self {
            main_table_state,
            current_encounter: value.current_encounter,
            current_panel: value.current_panel,
            editor_state: EditorState::default(),
            dirty: true,
        };
        app.sync_table_state();
        app
    }
}

// The side-effect the mainloop has to perform after App::update()
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Effect {
    None,
    Quit,
}

impl App {
    pub fn sync_table_state(&mut self) {
        if self.current_encounter.creatures.is_empty() {
            self.main_table_state.select(None);
        } else {
            let initiative_index = self
                .current_encounter
                .initiative_index
                .min(self.current_encounter.creatures.len().saturating_sub(1));
            self.main_table_state.select(Some(initiative_index));
        }
    }

    /// Add a creature to the state
    fn add_creature(&mut self, val: Creature) {
        self.current_encounter.add_creature(val);
        self.sync_table_state();
    }

    /// Select the next creature row for viewing
    fn select_next_row(&mut self) {
        self.current_encounter.select_next_cursor();
    }

    /// Select the previous creature row for viewing
    fn select_previous_row(&mut self) {
        self.current_encounter.select_previous_cursor();
    }

    fn increment_initiative_order(&mut self) {
        self.current_encounter.select_next_initiative();
        self.sync_table_state();
    }

    fn select_panel(&mut self, panel: Panel) {
        self.current_panel = panel;
    }

    fn submit_editor(&mut self) {
        todo!();
    }

    fn swap_panel(&mut self) {
        self.current_panel = match self.current_panel {
            Panel::InitiativeTable => Panel::Sidebar,
            Panel::Sidebar => Panel::InitiativeTable,
            _ => self.current_panel,
        }
    }

    fn delegate_editor_input_event(&mut self, e: &Event) {
        match self.editor_state.active_input {
            EditorField::Name => {
                self.editor_state.name_input.handle_event(&e);
            }
            EditorField::CurrentHP => {
                self.editor_state.cur_hp_input.handle_event(&e);
            }
            EditorField::MaxHP => {
                self.editor_state.max_hp_input.handle_event(&e);
            }
            EditorField::AC => {
                self.editor_state.ac_input.handle_event(&e);
            }
            EditorField::CR => {
                self.editor_state.cr_input.handle_event(&e);
            }
            EditorField::Amount => {
                self.editor_state.amount_input.handle_event(&e);
            }
            _ => (),
        }
    }

    pub fn update(&mut self, action: Action) -> Effect {
        match action {
            Action::SelectNextRow => {
                self.select_next_row();
                Effect::None
            }
            Action::SelectPreviousRow => {
                self.select_previous_row();
                Effect::None
            }
            Action::AdvanceTurn => {
                self.increment_initiative_order();
                Effect::None
            }
            Action::SwitchPanel => {
                self.swap_panel();
                Effect::None
            }
            Action::OpenEditor => {
                self.current_panel = Panel::Editor;
                Effect::None
            }
            Action::CloseEditor => {
                // TODO: Clear input states
                self.current_panel = Panel::InitiativeTable;
                Effect::None
            }
            Action::EditorNextField => {
                self.editor_state.next_field();
                Effect::None
            }
            Action::EditorPrevField => {
                self.editor_state.previous_field();
                Effect::None
            }
            Action::SubmitEditor => todo!(),
            Action::EditorInput(e) => {
                self.delegate_editor_input_event(&e);
                Effect::None
            }
            Action::Quit => Effect::Quit,
        }
    }
}

impl Serialize for App {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializableApp::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for App {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        SerializableApp::deserialize(deserializer).map(Into::into)
    }
}

impl Default for App {
    fn default() -> Self {
        let mut app = App {
            main_table_state: TableState::default(),
            current_encounter: Encounter::default(),
            current_panel: Panel::InitiativeTable,
            editor_state: EditorState::default(),
            dirty: true,
        };
        app.sync_table_state();
        app
    }
}

#[cfg(test)]
mod tests {
    use super::{App, Panel};
    use crate::model::Creature;

    #[test]
    fn app_serde_round_trips_encounter_state() {
        let mut app = App::default();
        app.select_panel(Panel::Editor);
        app.current_encounter
            .add_creature(Creature::new_player("Alice", 10, 10, None, None, None));
        app.current_encounter
            .add_creature(Creature::new_player("Bob", 10, 10, None, None, None));
        app.current_encounter.initiative_index = 1;
        app.sync_table_state();

        let json = serde_json::to_string(&app).unwrap();
        let mut restored: App = serde_json::from_str(&json).unwrap();
        restored.sync_table_state();

        assert_eq!(restored.current_encounter.initiative_index, 1);
        assert_eq!(restored.main_table_state.selected(), Some(1));
        assert_eq!(restored.current_encounter.creatures.len(), 2);
        assert_eq!(restored.current_panel, Panel::Editor);
    }
}
