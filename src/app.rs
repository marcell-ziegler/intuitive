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

/// Validation rule for a single editor field.
#[derive(Clone, Copy, Debug)]
pub enum FieldKind {
    /// Free text that must not be empty.
    Name,
    /// Unsigned integer within an inclusive range.
    Uint { min: u32, max: u32 },
    /// A number (possibly fractional) within an inclusive range.
    Number { min: f64, max: f64 },
}

/// A single text field in the editor: the text buffer, its validation rule, and
/// whether its current contents are valid.
///
/// `valid` starts `true` and is only flipped by [`EditorInput::validate`], so a
/// freshly opened editor shows no red borders until something is actually
/// checked (on keystroke for the edited field, on submit for all of them).
#[derive(Clone, Debug)]
pub struct EditorInput {
    pub input: Input,
    pub kind: FieldKind,
    pub valid: bool,
}

impl EditorInput {
    fn new(kind: FieldKind) -> Self {
        Self {
            input: Input::default(),
            kind,
            valid: true,
        }
    }

    /// Re-check the current contents against this field's [`FieldKind`], update
    /// `valid`, and return it. Bounds are inclusive.
    pub fn validate(&mut self) -> bool {
        let value = self.input.value().trim();
        self.valid = match self.kind {
            FieldKind::Name => !value.is_empty(),
            FieldKind::Uint { min, max } => {
                matches!(value.parse::<u32>(), Ok(n) if (min..=max).contains(&n))
            }
            FieldKind::Number { min, max } => {
                matches!(value.parse::<f64>(), Ok(n) if n >= min && n <= max)
            }
        };
        self.valid
    }
}

#[derive(Clone, Debug)]
pub struct EditorState {
    pub name: EditorInput,
    pub cur_hp: EditorInput,
    pub max_hp: EditorInput,
    pub ac: EditorInput,
    pub cr: EditorInput,
    pub amount: EditorInput,
    pub active_input: EditorField,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            name: EditorInput::new(FieldKind::Name),
            cur_hp: EditorInput::new(FieldKind::Uint {
                min: 0,
                max: 1_000_000,
            }),
            max_hp: EditorInput::new(FieldKind::Uint {
                min: 1,
                max: 1_000_000,
            }),
            ac: EditorInput::new(FieldKind::Uint { min: 0, max: 99 }),
            cr: EditorInput::new(FieldKind::Number {
                min: 0.0,
                max: 30.0,
            }),
            amount: EditorInput::new(FieldKind::Uint { min: 1, max: 99 }),
            active_input: EditorField::default(),
        }
    }
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

    /// All fields in display order, for operations over every input at once
    /// (validation, reset, ...). New fields must be added here.
    fn fields_mut(&mut self) -> [&mut EditorInput; 6] {
        [
            &mut self.name,
            &mut self.cur_hp,
            &mut self.max_hp,
            &mut self.ac,
            &mut self.cr,
            &mut self.amount,
        ]
    }

    /// The field the cursor is currently on, if any.
    fn active_field_mut(&mut self) -> Option<&mut EditorInput> {
        match self.active_input {
            EditorField::Name => Some(&mut self.name),
            EditorField::CurrentHP => Some(&mut self.cur_hp),
            EditorField::MaxHP => Some(&mut self.max_hp),
            EditorField::AC => Some(&mut self.ac),
            EditorField::CR => Some(&mut self.cr),
            EditorField::Amount => Some(&mut self.amount),
            EditorField::Unfocused => None,
        }
    }

    /// Feed an input event to the active field and re-validate it, so the red
    /// border clears as soon as the value becomes valid again.
    pub fn handle_input_event(&mut self, event: &Event) {
        if let Some(field) = self.active_field_mut() {
            field.input.handle_event(event);
            field.validate();
        }
    }

    pub fn clear(&mut self) {
        for field in self.fields_mut() {
            field.input.reset();
            field.valid = true;
        }
    }

    pub fn load_creature(&mut self, creature: &Creature) {
        self.name.input = self
            .name
            .input
            .clone()
            .with_value(creature.name().to_owned());
        self.max_hp.input = self
            .max_hp
            .input
            .clone()
            .with_value(creature.max_hp().to_string());
        self.cur_hp.input = self
            .cur_hp
            .input
            .clone()
            .with_value(creature.hp().to_string());
        self.ac.input = self.ac.input.clone().with_value(creature.ac().to_string());
        self.cr.input = self
            .cr
            .input
            .clone()
            .with_value(creature.get_level_or_cr().to_string());
        self.amount.input = self.amount.input.clone().with_value("1".into());
        for field in self.fields_mut() {
            field.valid = true;
        }
    }

    /// Validate every field against its [`FieldKind`], plus the cross-field rule
    /// that current HP may not exceed max HP. Sets each field's `valid` flag and
    /// returns `true` iff all inputs are valid.
    pub fn validate_inputs(&mut self) -> bool {
        let mut all_valid = true;
        for field in self.fields_mut() {
            if !field.validate() {
                all_valid = false;
            }
        }
        // Cross-field rule: current HP must not exceed max HP.
        if let (Ok(cur), Ok(max)) = (
            self.cur_hp.input.value().trim().parse::<u32>(),
            self.max_hp.input.value().trim().parse::<u32>(),
        ) && cur > max
        {
            self.cur_hp.valid = false;
            all_valid = false;
        }
        all_valid
    }

    /// Parse the fields into their typed values `(name, max_hp, cur_hp, ac, cr)`.
    /// Intended to be called after [`EditorState::validate_inputs`]; falls back
    /// to safe defaults for any field that fails to parse.
    fn parsed(&self) -> (String, u32, u32, u32, f64) {
        let name = self.name.input.value().trim().to_string();
        let max_hp = self.max_hp.input.value().trim().parse::<u32>().unwrap_or(1);
        let cur_hp = self
            .cur_hp
            .input
            .value()
            .trim()
            .parse::<u32>()
            .unwrap_or(max_hp);
        let ac = self.ac.input.value().trim().parse::<u32>().unwrap_or(10);
        let cr = self.cr.input.value().trim().parse::<f64>().unwrap_or(0.0);
        (name, max_hp, cur_hp, ac, cr)
    }

    /// The number of creatures to spawn (the Amount field), at least 1.
    fn amount(&self) -> u32 {
        self.amount
            .input
            .value()
            .trim()
            .parse::<u32>()
            .unwrap_or(1)
            .max(1)
    }

    /// Build a single [`Creature`] from the current field values.
    ///
    /// The editor has no Player/Monster toggle yet, so this always produces a
    /// [`Creature::Monster`].
    pub fn to_creature(&self) -> Creature {
        let (name, max_hp, cur_hp, ac, cr) = self.parsed();
        Creature::new_monster(&name, max_hp, ac, Some(cur_hp), None, Some(cr))
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
    UpdateState,
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

    fn delegate_editor_input_event(&mut self, e: &Event) {
        self.editor_state.handle_input_event(e);
    }

    pub fn update(&mut self, action: Action) -> Effect {
        match action {
            Action::SelectNextRow => {
                self.select_next_row();
                self.dirty = true;
                Effect::UpdateState
            }
            Action::SelectPreviousRow => {
                self.select_previous_row();
                self.dirty = true;
                Effect::UpdateState
            }
            Action::AdvanceTurn => {
                self.increment_initiative_order();
                self.dirty = true;
                Effect::UpdateState
            }
            Action::SwitchPanel => {
                self.swap_panel();
                self.dirty = true;
                Effect::None
            }
            Action::OpenEditorWithNewCreature => {
                self.editor_state.clear();
                self.current_panel = Panel::Editor;
                self.dirty = true;
                Effect::None
            }
            Action::OpenEditorAtIndex(index) => {
                if let Some(creature) = self.current_encounter.creatures.get(index as usize) {
                    let creature = creature.clone();
                    self.editor_state.load_creature(&creature);
                }
                self.current_panel = Panel::Editor;
                self.dirty = true;
                Effect::None
            }
            Action::CloseEditor => {
                self.editor_state.clear();
                self.dirty = true;
                self.current_panel = Panel::InitiativeTable;
                Effect::None
            }
            Action::EditorNextField => {
                self.editor_state.next_field();
                self.dirty = true;
                Effect::None
            }
            Action::EditorPrevField => {
                self.editor_state.previous_field();
                self.dirty = true;
                Effect::None
            }
            Action::SubmitEditor => {
                if self.submit_editor() {
                    self.dirty = true;
                    Effect::UpdateState
                } else {
                    // Validation failed — stay in the editor and redraw so the
                    // invalid fields show their red borders.
                    self.dirty = true;
                    Effect::None
                }
            }
            Action::EditorInput(e) => {
                self.delegate_editor_input_event(&e);
                self.dirty = true;
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
