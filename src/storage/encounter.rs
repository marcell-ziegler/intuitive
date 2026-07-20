use serde::{Deserialize, Serialize};

use crate::model::Creature;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Encounter {
    pub name: String,
    pub creatures: Vec<Creature>,
    pub initiative_index: usize,
    pub cursor_index: usize,
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

    /// Pull `cursor_index`/`initiative_index` back in bounds (0 if empty,
    /// otherwise at most the last row). The `select_*` methods above always
    /// keep both valid on their own via wrapping arithmetic, but a persisted
    /// `state.json` is untrusted input — a stale save from before rows were
    /// removed, or a hand-edited file, can carry an index the current
    /// `creatures` list no longer has. Call this once after loading.
    pub fn clamp_indices(&mut self) {
        let max = self.creatures.len().saturating_sub(1);
        self.cursor_index = self.cursor_index.min(max);
        self.initiative_index = self.initiative_index.min(max);
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Creature;
    use crate::storage::Encounter;

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

    #[test]
    fn clamp_indices_pulls_stale_indices_back_in_bounds() {
        let mut encounter = Encounter::default();
        encounter.add_creature(Creature::new_player("Alice", 10, 10, None, None, None));
        encounter.cursor_index = 5;
        encounter.initiative_index = 5;

        encounter.clamp_indices();

        assert_eq!(encounter.cursor_index, 0);
        assert_eq!(encounter.initiative_index, 0);
    }

    #[test]
    fn clamp_indices_on_empty_encounter_is_zero() {
        let mut encounter = Encounter::default();
        encounter.cursor_index = 3;
        encounter.initiative_index = 3;

        encounter.clamp_indices();

        assert_eq!(encounter.cursor_index, 0);
        assert_eq!(encounter.initiative_index, 0);
    }
}
