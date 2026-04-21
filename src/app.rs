use ratatui::widgets::TableState;
use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeStruct};

use crate::{model::Creature, model::Encounter};

#[derive(Debug, Clone)]
pub struct App {
    pub creatures: Vec<Creature>,
    pub main_table_state: TableState,
    pub selected_row: usize,
    pub current_encounter: Encounter,
}

#[derive(Debug, Clone, Deserialize)]
struct SerializableApp {
    creatures: Vec<Creature>,
    main_table_state_offset: usize,
    main_table_state_selected_column: usize,
    selected_row: usize,
    current_encounter: Encounter,
}

impl From<&App> for SerializableApp {
    fn from(app: &App) -> Self {
        Self {
            creatures: app.creatures.clone(),
            main_table_state_offset: app.main_table_state.offset(),
            main_table_state_selected_column: app
                .main_table_state
                .selected_column()
                .unwrap_or_default(),
            selected_row: app.selected_row,
            current_encounter: app.current_encounter.clone(),
        }
    }
}

impl From<SerializableApp> for App {
    fn from(value: SerializableApp) -> Self {
        let mut main_table_state = TableState::default();
        main_table_state.select_column(Some(value.main_table_state_selected_column));
        *main_table_state.offset_mut() = value.main_table_state_offset;

        Self {
            creatures: value.creatures,
            main_table_state,
            selected_row: value.selected_row,
            current_encounter: value.current_encounter,
        }
    }
}

impl App {
    /// Add a creature to the state
    pub fn add_creature(&mut self, val: Creature) {
        self.creatures.push(val);
    }

    /// Select the next creature row for viewing
    pub fn select_next_row(&mut self) {
        if self.selected_row >= self.creatures.len() - 1 {
            self.selected_row = 0;
        } else {
            self.selected_row += 1;
        }
    }

    /// Select the previous creature row for viewing
    pub fn select_previous_row(&mut self) {
        if self.selected_row <= 0 {
            self.selected_row = (self.creatures.len() - 1).max(0);
        } else {
            self.selected_row -= 1;
        }
    }

    pub fn increment_initiative_order(&mut self) {
        let len = self.creatures.len();
        if len == 0 {
            self.main_table_state.select(None);
            return;
        }

        let next = match self.main_table_state.selected() {
            Some(i) => (i + 1) % len,
            None => 0,
        };

        self.main_table_state.select(Some(next));
    }
}

impl Serialize for App {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("App", 5)?;
        state.serialize_field("creatures", &self.creatures)?;
        state.serialize_field("main_table_state_offset", &self.main_table_state.offset())?;
        state.serialize_field(
            "main_table_state_selected_column",
            &self.main_table_state.selected_column(),
        )?;
        state.serialize_field("selected_row", &self.selected_row)?;
        state.serialize_field("current_encounter", &self.current_encounter)?;
        state.end()
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
        let mut main_table_state = TableState::new();
        main_table_state.select_first();
        main_table_state.select_first_column();
        App {
            creatures: Vec::new(),
            main_table_state,
            selected_row: 0,
            current_encounter: Encounter::default(),
        }
    }
}
