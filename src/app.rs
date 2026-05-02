use ratatui::widgets::TableState;
use ratatui_textarea::TextArea;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tui_input::Input;

use crate::{model::Creature, model::Encounter};

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
}

#[derive(Clone, Debug)]
pub struct EditorState {
    pub name_input: Input,
    pub max_hp: u32,
    pub cur_hp: u32,
    pub ac: u32,
    pub cr: f64,
    pub active_input: EditorField,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum EditorField {
    Name,
    MaxHP,
    CurrentHP,
    AC,
    CR,
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
        };
        app.sync_table_state();
        app
    }
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
    pub fn add_creature(&mut self, val: Creature) {
        self.current_encounter.add_creature(val);
        self.sync_table_state();
    }

    /// Select the next creature row for viewing
    pub fn select_next_row(&mut self) {
        self.current_encounter.select_next_cursor();
    }

    /// Select the previous creature row for viewing
    pub fn select_previous_row(&mut self) {
        self.current_encounter.select_previous_cursor();
    }

    pub fn increment_initiative_order(&mut self) {
        self.current_encounter.select_next_initiative();
        self.sync_table_state();
    }

    pub fn select_panel(&mut self, panel: Panel) {
        self.current_panel = panel;
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
