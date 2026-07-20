use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    action::Action,
    editor::EditorState,
    model::{Creature, Encounter},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Panel {
    InitiativeTable,
    Sidebar,
    Editor,
}

// The side-effect the mainloop has to perform after App::update()
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Effect {
    None,
    Quit,
    UpdateState,
}

#[derive(Debug, Clone)]
pub struct App {
    pub current_encounter: Encounter,
    pub current_panel: Panel,
    pub editor_state: EditorState,
    /// Set on any mutation that should eventually be persisted but doesn't need an immediate save (e.g. cursor movement).
    pub state_dirty: bool,
}

impl App {
    /// Add a creature to the state
    fn add_creature(&mut self, val: Creature) {
        self.current_encounter.add_creature(val);
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
    }

    fn select_panel(&mut self, panel: Panel) {
        self.current_panel = panel;
    }

    /// Validate the editor and, if valid, add the resulting creature(s) to the
    /// encounter (the Amount field spawns numbered copies), then close the
    /// editor. Returns `false` (leaving the editor open) if validation failed.
    fn submit_editor(&mut self) -> bool {
        if !self.editor_state.validate_inputs() {
            return false;
        }

        let amount = self.editor_state.amount();
        if amount == 1 {
            let creature = self.editor_state.to_creature();
            self.add_creature(creature);
        } else {
            let (name, max_hp, cur_hp, ac, cr) = self.editor_state.parsed();
            for i in 1..=amount {
                let creature = Creature::new_monster(
                    &format!("{name} {i}"),
                    max_hp,
                    ac,
                    Some(cur_hp),
                    None,
                    Some(cr),
                );
                self.add_creature(creature);
            }
        }

        self.editor_state.clear();
        self.current_panel = Panel::InitiativeTable;
        true
    }

    fn swap_panel(&mut self) {
        self.current_panel = match self.current_panel {
            Panel::InitiativeTable => Panel::Sidebar,
            Panel::Sidebar => Panel::InitiativeTable,
            _ => self.current_panel,
        }
    }

    pub fn update(&mut self, action: Action) -> Effect {
        match action {
            Action::SelectNextRow => {
                self.select_next_row();
                self.state_dirty = true;
                Effect::None
            }
            Action::SelectPreviousRow => {
                self.select_previous_row();
                self.state_dirty = true;
                Effect::None
            }
            Action::AdvanceTurn => {
                self.increment_initiative_order();
                self.state_dirty = true;
                Effect::None
            }
            Action::SwitchPanel => {
                self.swap_panel();
                Effect::None
            }
            Action::OpenEditorWithNewCreature => {
                self.editor_state.clear();
                self.current_panel = Panel::Editor;
                Effect::None
            }
            Action::OpenEditorAtIndex(index) => {
                if let Some(creature) = self.current_encounter.creatures.get(index as usize) {
                    let creature = creature.clone();
                    self.editor_state.load_creature(&creature);
                }
                self.current_panel = Panel::Editor;
                Effect::None
            }
            Action::CloseEditor => {
                self.editor_state.clear();
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
            Action::SubmitEditor => {
                if self.submit_editor() {
                    Effect::UpdateState
                } else {
                    // Validation failed — stay in the editor and redraw so the
                    // invalid fields show their red borders.
                    Effect::None
                }
            }
            Action::EditorInput(e) => {
                self.editor_state.handle_input_event(&e);
                Effect::None
            }
            Action::Quit => Effect::Quit,
        }
    }
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
        let mut current_encounter = value.current_encounter;
        current_encounter.clamp_indices();
        Self {
            current_encounter,
            current_panel: value.current_panel,
            editor_state: EditorState::default(),
            state_dirty: false,
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
        Self {
            current_encounter: Encounter::default(),
            current_panel: Panel::InitiativeTable,
            editor_state: EditorState::default(),
            state_dirty: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{App, EditorState, Panel};
    use crate::model::Creature;

    /// Fill every editor field with a valid value.
    fn valid_editor() -> EditorState {
        let mut e = EditorState::default();
        e.name.input = e.name.input.clone().with_value("Goblin".into());
        e.cur_hp.input = e.cur_hp.input.clone().with_value("7".into());
        e.max_hp.input = e.max_hp.input.clone().with_value("7".into());
        e.ac.input = e.ac.input.clone().with_value("15".into());
        e.cr.input = e.cr.input.clone().with_value("0.25".into());
        e.amount.input = e.amount.input.clone().with_value("1".into());
        e
    }

    #[test]
    fn validate_accepts_well_formed_inputs() {
        let mut e = valid_editor();
        assert!(e.validate_inputs());
        assert!(e.name.valid && e.cur_hp.valid && e.max_hp.valid);
        assert!(e.ac.valid && e.cr.valid && e.amount.valid);
    }

    #[test]
    fn validate_rejects_empty_name_and_non_numeric_hp() {
        let mut e = valid_editor();
        e.name.input = e.name.input.clone().with_value("".into());
        e.max_hp.input = e.max_hp.input.clone().with_value("lots".into());

        assert!(!e.validate_inputs());
        assert!(!e.name.valid);
        assert!(!e.max_hp.valid);
        assert!(e.ac.valid); // untouched valid fields stay valid
    }

    #[test]
    fn validate_bounds_checks_ac_and_cr() {
        let mut e = valid_editor();
        e.ac.input = e.ac.input.clone().with_value("999".into()); // above max
        e.cr.input = e.cr.input.clone().with_value("40".into()); // above max

        assert!(!e.validate_inputs());
        assert!(!e.ac.valid);
        assert!(!e.cr.valid);
    }

    #[test]
    fn validate_rejects_current_hp_above_max() {
        let mut e = valid_editor();
        e.cur_hp.input = e.cur_hp.input.clone().with_value("20".into());
        e.max_hp.input = e.max_hp.input.clone().with_value("7".into());

        assert!(!e.validate_inputs());
        assert!(!e.cur_hp.valid);
        assert!(e.max_hp.valid); // max itself is fine
    }

    #[test]
    fn to_creature_builds_a_monster_from_fields() {
        let creature = valid_editor().to_creature();
        match creature {
            Creature::Monster { .. } => {}
            _ => panic!("expected a Monster"),
        }
        assert_eq!(creature.name(), "Goblin");
        assert_eq!(creature.max_hp(), 7);
        assert_eq!(creature.hp(), 7);
        assert_eq!(creature.ac(), 15);
        assert_eq!(creature.get_level_or_cr(), 0.25);
    }

    #[test]
    fn app_serde_round_trips_encounter_state() {
        let mut app = App::default();
        app.select_panel(Panel::Editor);
        app.current_encounter
            .add_creature(Creature::new_player("Alice", 10, 10, None, None, None));
        app.current_encounter
            .add_creature(Creature::new_player("Bob", 10, 10, None, None, None));
        app.current_encounter.initiative_index = 1;

        let json = serde_json::to_string(&app).unwrap();
        let restored: App = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.current_encounter.initiative_index, 1);
        assert_eq!(restored.current_encounter.creatures.len(), 2);
        assert_eq!(restored.current_panel, Panel::Editor);
    }
}
