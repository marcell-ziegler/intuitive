use ratatui::widgets::TableState;

use crate::model::Creature;

pub struct App {
    pub creatures: Vec<Creature>,
    pub main_table_state: TableState,
    pub selected_row: usize,
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

impl Default for App {
    fn default() -> Self {
        let mut main_table_state = TableState::new();
        main_table_state.select_first();
        main_table_state.select_first_column();
        App {
            creatures: Vec::new(),
            main_table_state,
            selected_row: 0,
        }
    }
}
