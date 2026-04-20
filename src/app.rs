use ratatui::widgets::TableState;

use crate::model::Creature;

pub struct App {
    pub creatures: Vec<Creature>,
    pub main_table_state: TableState,
}

impl App {
    /// Add a creature to the state
    pub fn add_creature(&mut self, val: Creature) {
        self.creatures.push(val);
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
        }
    }
}
