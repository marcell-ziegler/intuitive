use serde::{Deserialize, Serialize};

use crate::model::Creature;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encounter {
    pub name: String,
    pub creatures: Vec<Creature>,
    pub initiative_index: usize,
    pub cursor_index: usize,
}

impl Default for Encounter {
    fn default() -> Self {
        Encounter {
            name: String::new(),
            creatures: vec![],
            initiative_index: 0,
            cursor_index: 0,
        }
    }
}

impl Encounter {
    pub fn add_creature(&mut self, creature: Creature) {
        self.creatures.push(creature);
        if self.creatures.len() == 1 {
            self.cursor_index = 0;
            self.initiative_index = 0;
        }
    }

    pub fn select_next_cursor(&mut self) {
        let len = self.creatures.len();
        if len == 0 {
            self.cursor_index = 0;
        } else {
            self.cursor_index = (self.cursor_index + 1) % len;
        }
    }

    pub fn select_previous_cursor(&mut self) {
        let len = self.creatures.len();
        if len == 0 {
            self.cursor_index = 0;
        } else if self.cursor_index == 0 {
            self.cursor_index = len - 1;
        } else {
            self.cursor_index -= 1;
        }
    }

    pub fn select_next_initiative(&mut self) {
        let len = self.creatures.len();
        if len == 0 {
            self.initiative_index = 0;
        } else {
            self.initiative_index = (self.initiative_index + 1) % len;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Creature, Encounter};

    #[test]
    fn cursor_wraps_in_both_directions() {
        let mut encounter = Encounter::default();
        encounter.add_creature(Creature::new_player("Alice", 10, 10, None, None, None));
        encounter.add_creature(Creature::new_player("Bob", 10, 10, None, None, None));

        assert_eq!(encounter.cursor_index, 0);
        encounter.select_next_cursor();
        assert_eq!(encounter.cursor_index, 1);
        encounter.select_next_cursor();
        assert_eq!(encounter.cursor_index, 0);
        encounter.select_previous_cursor();
        assert_eq!(encounter.cursor_index, 1);
    }
}
