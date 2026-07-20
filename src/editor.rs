use crossterm::event::Event;
use serde::{Deserialize, Serialize};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::model::Creature;

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
    pub fn parsed(&self) -> (String, u32, u32, u32, f64) {
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
    pub fn amount(&self) -> u32 {
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
