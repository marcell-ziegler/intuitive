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
            let (name, _, _, _, _) = self.editor_state.parsed();
            for i in 1..=amount {
                let creature = self.editor_state.creature_with_name(&format!("{name} {i}"));
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
            Action::EditorToggleCreatureType => {
                self.editor_state.toggle_creature_type();
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
    use super::{App, EditorState, Effect, Panel};
    use crate::action::Action;
    use crate::editor::EditorField;
    use crate::model::Creature;
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

    /// An `App` holding `n` throwaway creatures, cursor/initiative at 0.
    fn app_with_creatures(n: usize) -> App {
        let mut app = App::default();
        for i in 0..n {
            app.current_encounter.add_creature(Creature::new_player(
                &format!("C{i}"),
                10,
                10,
                None,
                None,
                None,
            ));
        }
        app
    }

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
    fn to_creature_builds_a_player_when_toggled() {
        let mut e = valid_editor();
        e.cr.input = e.cr.input.clone().with_value("5".into()); // reused as level
        e.toggle_creature_type();

        let creature = e.to_creature();
        match creature {
            Creature::Player { .. } => {}
            _ => panic!("expected a Player"),
        }
        assert_eq!(creature.name(), "Goblin");
        assert_eq!(creature.get_level_or_cr(), 5.0);
    }

    #[test]
    fn toggle_creature_type_flips_back_and_forth() {
        let mut e = EditorState::default();
        assert_eq!(e.creature_type, crate::editor::CreatureType::Monster);
        e.toggle_creature_type();
        assert_eq!(e.creature_type, crate::editor::CreatureType::Player);
        e.toggle_creature_type();
        assert_eq!(e.creature_type, crate::editor::CreatureType::Monster);
    }

    #[test]
    fn load_creature_sets_creature_type_from_the_loaded_creature() {
        let mut e = EditorState::default();
        e.toggle_creature_type(); // start on Player, to prove load_creature overwrites it
        e.load_creature(&Creature::new_monster(
            "Goblin",
            7,
            15,
            Some(7),
            None,
            Some(0.25),
        ));
        assert_eq!(e.creature_type, crate::editor::CreatureType::Monster);
    }

    #[test]
    fn clear_resets_creature_type_to_monster() {
        let mut e = valid_editor();
        e.toggle_creature_type();
        e.clear();
        assert_eq!(e.creature_type, crate::editor::CreatureType::Monster);
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

    // --- `App::update` — one test per `Action`, asserting the returned `Effect`
    //     and the resulting state (Phase 0, Step 7). ---

    #[test]
    fn update_select_next_row_advances_cursor_and_marks_dirty() {
        let mut app = app_with_creatures(2);
        assert_eq!(app.update(Action::SelectNextRow), Effect::None);
        assert_eq!(app.current_encounter.cursor_index, 1);
        assert!(app.state_dirty);
    }

    #[test]
    fn update_select_previous_row_wraps_cursor_and_marks_dirty() {
        let mut app = app_with_creatures(2);
        assert_eq!(app.update(Action::SelectPreviousRow), Effect::None);
        assert_eq!(app.current_encounter.cursor_index, 1); // 0 wraps to last row
        assert!(app.state_dirty);
    }

    #[test]
    fn update_advance_turn_moves_initiative_and_marks_dirty() {
        let mut app = app_with_creatures(2);
        assert_eq!(app.update(Action::AdvanceTurn), Effect::None);
        assert_eq!(app.current_encounter.initiative_index, 1);
        assert!(app.state_dirty);
    }

    #[test]
    fn update_switch_panel_toggles_and_stays_clean() {
        let mut app = App::default();
        assert_eq!(app.current_panel, Panel::InitiativeTable);
        assert_eq!(app.update(Action::SwitchPanel), Effect::None);
        assert_eq!(app.current_panel, Panel::Sidebar);
        app.update(Action::SwitchPanel);
        assert_eq!(app.current_panel, Panel::InitiativeTable);
        assert!(!app.state_dirty); // panel is view-only, not persisted
    }

    #[test]
    fn update_open_editor_with_new_creature_opens_blank_editor() {
        let mut app = App::default();
        app.editor_state.name.input = app
            .editor_state
            .name
            .input
            .clone()
            .with_value("stale".into());
        assert_eq!(app.update(Action::OpenEditorWithNewCreature), Effect::None);
        assert_eq!(app.current_panel, Panel::Editor);
        assert_eq!(app.editor_state.name.input.value(), ""); // cleared
    }

    #[test]
    fn update_open_editor_at_index_loads_the_creature() {
        let mut app = App::default();
        app.current_encounter.add_creature(Creature::new_monster(
            "Goblin",
            7,
            15,
            Some(7),
            None,
            Some(0.25),
        ));
        assert_eq!(app.update(Action::OpenEditorAtIndex(0)), Effect::None);
        assert_eq!(app.current_panel, Panel::Editor);
        assert_eq!(app.editor_state.name.input.value(), "Goblin");
        assert_eq!(app.editor_state.ac.input.value(), "15");
    }

    #[test]
    fn update_open_editor_at_out_of_range_index_opens_without_loading() {
        let mut app = App::default();
        assert_eq!(app.update(Action::OpenEditorAtIndex(9)), Effect::None);
        assert_eq!(app.current_panel, Panel::Editor);
        assert_eq!(app.editor_state.name.input.value(), "");
    }

    #[test]
    fn update_close_editor_returns_to_table_and_clears() {
        let mut app = App::default();
        app.current_panel = Panel::Editor;
        app.editor_state.name.input = app
            .editor_state
            .name
            .input
            .clone()
            .with_value("half".into());
        assert_eq!(app.update(Action::CloseEditor), Effect::None);
        assert_eq!(app.current_panel, Panel::InitiativeTable);
        assert_eq!(app.editor_state.name.input.value(), "");
    }

    #[test]
    fn update_editor_field_navigation_moves_focus() {
        let mut app = App::default();
        app.current_panel = Panel::Editor;
        assert_eq!(app.editor_state.active_input, EditorField::Name);
        assert_eq!(app.update(Action::EditorNextField), Effect::None);
        assert_eq!(app.editor_state.active_input, EditorField::CurrentHP);
        assert_eq!(app.update(Action::EditorPrevField), Effect::None);
        assert_eq!(app.editor_state.active_input, EditorField::Name);
    }

    #[test]
    fn update_editor_toggle_creature_type_flips_the_type() {
        let mut app = App::default();
        app.current_panel = Panel::Editor;
        assert_eq!(
            app.editor_state.creature_type,
            crate::editor::CreatureType::Monster
        );
        assert_eq!(app.update(Action::EditorToggleCreatureType), Effect::None);
        assert_eq!(
            app.editor_state.creature_type,
            crate::editor::CreatureType::Player
        );
    }

    #[test]
    fn update_submit_valid_editor_adds_creature_and_requests_save() {
        let mut app = App::default();
        app.current_panel = Panel::Editor;
        app.editor_state = valid_editor();
        assert_eq!(app.update(Action::SubmitEditor), Effect::UpdateState);
        assert_eq!(app.current_encounter.creatures.len(), 1);
        assert_eq!(app.current_encounter.creatures[0].name(), "Goblin");
        assert_eq!(app.current_panel, Panel::InitiativeTable); // editor closed
    }

    #[test]
    fn update_submit_invalid_editor_keeps_editor_open_and_adds_nothing() {
        let mut app = App::default();
        app.current_panel = Panel::Editor;
        app.editor_state = valid_editor();
        app.editor_state.name.input = app.editor_state.name.input.clone().with_value("".into());
        assert_eq!(app.update(Action::SubmitEditor), Effect::None);
        assert_eq!(app.current_encounter.creatures.len(), 0);
        assert_eq!(app.current_panel, Panel::Editor); // stayed open
    }

    #[test]
    fn update_submit_with_amount_spawns_numbered_copies() {
        let mut app = App::default();
        app.current_panel = Panel::Editor;
        app.editor_state = valid_editor();
        app.editor_state.amount.input =
            app.editor_state.amount.input.clone().with_value("3".into());
        assert_eq!(app.update(Action::SubmitEditor), Effect::UpdateState);
        assert_eq!(app.current_encounter.creatures.len(), 3);
        assert_eq!(app.current_encounter.creatures[0].name(), "Goblin 1");
        assert_eq!(app.current_encounter.creatures[2].name(), "Goblin 3");
    }

    #[test]
    fn update_editor_input_is_delegated_to_the_active_field() {
        let mut app = App::default();
        app.current_panel = Panel::Editor; // active field defaults to Name
        let ev = Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty()));
        assert_eq!(app.update(Action::EditorInput(ev)), Effect::None);
        assert_eq!(app.editor_state.name.input.value(), "x");
    }

    #[test]
    fn update_quit_returns_quit_effect() {
        let mut app = App::default();
        assert_eq!(app.update(Action::Quit), Effect::Quit);
    }
}
