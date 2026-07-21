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

    /// Remove the creature at `index`, if present, and pull `cursor_index`/
    /// `initiative_index` back in bounds afterward (the removal can shrink
    /// `creatures` out from under whichever index they were pointing at).
    /// No-ops and returns `None` for an out-of-range index rather than
    /// panicking, matching `creatures.get()`'s behavior elsewhere.
    pub fn remove_creature(&mut self, index: usize) -> Option<Creature> {
        if index >= self.creatures.len() {
            return None;
        }
        let removed = self.creatures.remove(index);
        self.clamp_indices();
        Some(removed)
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

    #[test]
    fn remove_creature_removes_and_shrinks_the_list() {
        let mut encounter = Encounter::default();
        encounter.add_creature(Creature::new_player("Alice", 10, 10, None, None, None));
        encounter.add_creature(Creature::new_player("Bob", 10, 10, None, None, None));
        encounter.add_creature(Creature::new_player("Carol", 10, 10, None, None, None));

        let removed = encounter.remove_creature(1);

        assert_eq!(removed.map(|c| c.name().to_string()), Some("Bob".into()));
        assert_eq!(encounter.creatures.len(), 2);
        assert_eq!(encounter.creatures[0].name(), "Alice");
        assert_eq!(encounter.creatures[1].name(), "Carol");
    }

    #[test]
    fn remove_creature_out_of_range_is_a_safe_no_op() {
        let mut encounter = Encounter::default();
        encounter.add_creature(Creature::new_player("Alice", 10, 10, None, None, None));

        assert!(encounter.remove_creature(5).is_none());
        assert_eq!(encounter.creatures.len(), 1);
    }

    #[test]
    fn remove_creature_on_empty_encounter_does_not_panic() {
        let mut encounter = Encounter::default();
        assert!(encounter.remove_creature(0).is_none());
    }

    #[test]
    fn remove_creature_clamps_stale_selection_indices() {
        let mut encounter = Encounter::default();
        encounter.add_creature(Creature::new_player("Alice", 10, 10, None, None, None));
        encounter.add_creature(Creature::new_player("Bob", 10, 10, None, None, None));
        encounter.cursor_index = 1;
        encounter.initiative_index = 1;

        // Removing the last row leaves cursor/initiative pointing past the end
        // until clamp_indices (called internally) pulls them back.
        encounter.remove_creature(1);

        assert_eq!(encounter.cursor_index, 0);
        assert_eq!(encounter.initiative_index, 0);
    }
}
