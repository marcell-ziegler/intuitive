use crossterm::event::{Event, KeyCode};
use dice_parser::DiceExpr;
use serde::{Deserialize, Serialize};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::model::{Creature, Encounter, Status};

/// Validation rule for a single editor field.
#[derive(Clone, Copy, Debug)]
pub enum FieldKind {
    /// Free text that must not be empty.
    Name,
    /// Unsigned integer within an inclusive range.
    Uint { min: u32, max: u32 },
    /// A number (possibly fractional) within an inclusive range.
    Number { min: f64, max: f64 },
    /// A dice-notation expression (e.g. `"2d8+2"`), or empty.
    DiceExpr,
    /// An unsigned integer within an inclusive range, or empty — unlike
    /// `Uint`, empty is valid here. Used for fields that represent an
    /// optional value (e.g. Initiative: not every creature has rolled it).
    OptionalUint { min: u32, max: u32 },
    /// The first word must name a `Status` variant (case-insensitive), or
    /// the field is empty. Doesn't check a Grappled target exists — that
    /// needs `&Encounter`, resolved separately at submit time.
    StatusName,
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
            FieldKind::DiceExpr => value.is_empty() || DiceExpr::parse(value).is_ok(),
            FieldKind::OptionalUint { min, max } => {
                value.is_empty()
                    || matches!(value.parse::<u32>(), Ok(n) if (min..=max).contains(&n))
            }
            FieldKind::StatusName => value.is_empty() || is_known_status_name(value),
        };
        self.valid
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CreatureType {
    Player,
    #[default]
    Monster,
}

impl CreatureType {
    pub fn toggle(&mut self) {
        *self = match self {
            CreatureType::Player => CreatureType::Monster,
            CreatureType::Monster => CreatureType::Player,
        };
    }
}

/// Whether a new creature's HP is rolled from its Hit Die or typed manually.
/// Always `Manual` while editing an existing creature — re-rolling an
/// existing creature's max HP doesn't make sense the way it does at creation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HpMode {
    Roll,
    #[default]
    Manual,
}

/// Match `text`'s leading word against the 15 `Status` variant names,
/// case-insensitively. Cheap — no `&Encounter` needed — used for live
/// validation on every keystroke; doesn't check a Grappled target resolves,
/// see [`parse_status`] for the full check used at submit time.
fn is_known_status_name(text: &str) -> bool {
    let word = text.split_whitespace().next().unwrap_or("");
    matches!(
        word.to_lowercase().as_str(),
        "blinded"
            | "charmed"
            | "deafened"
            | "exhaustion"
            | "frightened"
            | "grappled"
            | "incapacitated"
            | "invisible"
            | "paralyzed"
            | "petrified"
            | "poisoned"
            | "prone"
            | "restrained"
            | "stunned"
            | "unconscious"
    )
}

/// Parse `text` into a `Status`. The first whitespace-separated word selects
/// the variant (case-insensitive); for `Grappled`, everything after it is
/// treated as the target creature's name and resolved against `encounter` —
/// an unresolvable name is rejected. `Exhaustion`'s count-bumping (typing it
/// again increases the count rather than duplicating) is handled by the
/// caller, not here — this always returns a fresh `Exhaustion(1)`.
fn parse_status(text: &str, encounter: &Encounter) -> Result<Status, ()> {
    let text = text.trim();
    let mut parts = text.splitn(2, char::is_whitespace);
    let name = parts.next().unwrap_or("").trim();
    let rest = parts.next().unwrap_or("").trim();
    match name.to_lowercase().as_str() {
        "blinded" => Ok(Status::Blinded),
        "charmed" => Ok(Status::Charmed),
        "deafened" => Ok(Status::Deafened),
        "exhaustion" => Ok(Status::Exhaustion(1)),
        "frightened" => Ok(Status::Frightened),
        "grappled" => encounter
            .creatures
            .iter()
            .find(|c| c.name().eq_ignore_ascii_case(rest))
            .map(|c| Status::Grappled(c.id()))
            .ok_or(()),
        "incapacitated" => Ok(Status::Incapacitated),
        "invisible" => Ok(Status::Invisible),
        "paralyzed" => Ok(Status::Paralyzed),
        "petrified" => Ok(Status::Petrified),
        "poisoned" => Ok(Status::Poisoned),
        "prone" => Ok(Status::Prone),
        "restrained" => Ok(Status::Restrained),
        "stunned" => Ok(Status::Stunned),
        "unconscious" => Ok(Status::Unconscious),
        _ => Err(()),
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
    pub initiative: EditorInput,
    pub hit_die: EditorInput,
    pub statuses_input: EditorInput,
    pub creature_type: CreatureType,
    pub hp_mode: HpMode,
    pub pending_statuses: Vec<Status>,
    pub selected_status_index: Option<usize>,
    pub active_input: EditorField,
    pub editing_index: Option<usize>,
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
            amount: EditorInput {
                input: Input::default().with_value("1".into()),
                kind: FieldKind::Uint { min: 1, max: 99 },
                valid: true,
            },
            initiative: EditorInput::new(FieldKind::OptionalUint { min: 0, max: 99 }),
            hit_die: EditorInput::new(FieldKind::DiceExpr),
            statuses_input: EditorInput::new(FieldKind::StatusName),
            active_input: EditorField::default(),
            creature_type: CreatureType::Monster,
            hp_mode: HpMode::default(),
            pending_statuses: Vec::new(),
            selected_status_index: None,
            editing_index: None,
        }
    }
}

impl EditorState {
    /// Tab-focusable fields in cycle order, filtered for the current mode:
    /// Current/Max HP are skipped while `hp_mode == Roll` (they're computed,
    /// not typed). A fixed match cascade can't express that conditional skip
    /// cleanly, hence the array + position-search below instead.
    fn focusable_fields(&self) -> Vec<EditorField> {
        let mut fields = vec![EditorField::Name];
        if self.hp_mode != HpMode::Roll {
            fields.push(EditorField::CurrentHP);
            fields.push(EditorField::MaxHP);
        }
        fields.push(EditorField::AC);
        fields.push(EditorField::CR);
        fields.push(EditorField::Amount);
        fields.push(EditorField::Initiative);
        fields.push(EditorField::HitDie);
        fields.push(EditorField::Statuses);
        fields
    }

    pub fn next_field(&mut self) {
        let fields = self.focusable_fields();
        let pos = fields.iter().position(|f| *f == self.active_input);
        self.active_input = match pos {
            Some(i) => fields[(i + 1) % fields.len()],
            None => fields[0],
        };
    }

    pub fn previous_field(&mut self) {
        let fields = self.focusable_fields();
        let pos = fields.iter().position(|f| *f == self.active_input);
        self.active_input = match pos {
            Some(0) | None => *fields.last().unwrap(),
            Some(i) => fields[i - 1],
        };
    }

    pub fn toggle_creature_type(&mut self) {
        self.creature_type.toggle();
    }

    /// Flip Roll/Manual HP mode. A no-op while editing an existing creature —
    /// re-rolling an existing creature's max HP doesn't make sense the way it
    /// does at creation.
    pub fn toggle_hp_mode(&mut self) {
        if self.editing_index.is_some() {
            return;
        }
        self.hp_mode = match self.hp_mode {
            HpMode::Roll => HpMode::Manual,
            HpMode::Manual => HpMode::Roll,
        };
    }

    /// All text-input widgets, for operations over every one at once
    /// (reset, mark-all-valid). New text fields must be added here.
    fn fields_mut(&mut self) -> [&mut EditorInput; 9] {
        [
            &mut self.name,
            &mut self.cur_hp,
            &mut self.max_hp,
            &mut self.ac,
            &mut self.cr,
            &mut self.amount,
            &mut self.initiative,
            &mut self.hit_die,
            &mut self.statuses_input,
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
            EditorField::Initiative => Some(&mut self.initiative),
            EditorField::HitDie => Some(&mut self.hit_die),
            EditorField::Statuses => Some(&mut self.statuses_input),
            EditorField::Unfocused => None,
        }
    }

    /// Feed an input event to the active field and re-validate it, so the red
    /// border clears as soon as the value becomes valid again.
    ///
    /// The Statuses field is special: Enter submits the typed text as a new
    /// status instead of being inserted as text, and — only when the input
    /// box is empty, so normal typing/erasing is never affected —
    /// Left/Right move the delete-selection and Backspace/Delete remove the
    /// selected status. Needs `&Encounter` to resolve a Grappled target by
    /// name.
    pub fn handle_input_event(&mut self, event: &Event, encounter: &Encounter) {
        if self.active_input == EditorField::Statuses
            && let Some(key_event) = event.as_key_event()
        {
            let box_empty = self.statuses_input.input.value().is_empty();
            match key_event.code {
                KeyCode::Enter => {
                    self.try_submit_status(encounter);
                    return;
                }
                KeyCode::Left if box_empty => {
                    self.select_previous_status();
                    return;
                }
                KeyCode::Right if box_empty => {
                    self.select_next_status();
                    return;
                }
                KeyCode::Backspace | KeyCode::Delete if box_empty => {
                    self.delete_selected_status();
                    return;
                }
                _ => {}
            }
        }

        if let Some(field) = self.active_field_mut() {
            field.input.handle_event(event);
            field.validate();
        }
    }

    /// Parse the Statuses box's current text and, on success, add it to
    /// `pending_statuses` (bumping the count instead of duplicating for a
    /// repeated `Exhaustion`) and clear the box. On failure — including an
    /// empty box — leaves the text as-is; a non-empty unparsable entry is
    /// marked invalid so its border turns red. Returns whether it succeeded.
    pub fn try_submit_status(&mut self, encounter: &Encounter) -> bool {
        let text = self.statuses_input.input.value().to_string();
        if text.trim().is_empty() {
            return false;
        }

        match parse_status(&text, encounter) {
            Ok(Status::Exhaustion(_)) => {
                let mut bumped = false;
                for s in self.pending_statuses.iter_mut() {
                    if let Status::Exhaustion(n) = s {
                        *n += 1;
                        bumped = true;
                        break;
                    }
                }
                if !bumped {
                    self.pending_statuses.push(Status::Exhaustion(1));
                }
                self.statuses_input.input.reset();
                self.statuses_input.valid = true;
                if self.selected_status_index.is_none() {
                    self.selected_status_index = Some(0);
                }
                true
            }
            Ok(status) => {
                self.pending_statuses.push(status);
                self.statuses_input.input.reset();
                self.statuses_input.valid = true;
                if self.selected_status_index.is_none() {
                    self.selected_status_index = Some(0);
                }
                true
            }
            Err(()) => {
                self.statuses_input.valid = false;
                false
            }
        }
    }

    fn select_previous_status(&mut self) {
        let len = self.pending_statuses.len();
        if len == 0 {
            self.selected_status_index = None;
            return;
        }
        self.selected_status_index = Some(match self.selected_status_index {
            Some(0) | None => len - 1,
            Some(i) => i - 1,
        });
    }

    fn select_next_status(&mut self) {
        let len = self.pending_statuses.len();
        if len == 0 {
            self.selected_status_index = None;
            return;
        }
        self.selected_status_index = Some(match self.selected_status_index {
            Some(i) => (i + 1) % len,
            None => 0,
        });
    }

    /// Remove the selected status — decrementing `Exhaustion(n)` to `n - 1`
    /// and only dropping the entry once it reaches 0, or removing any other
    /// status outright.
    fn delete_selected_status(&mut self) {
        let Some(idx) = self.selected_status_index else {
            return;
        };
        let Some(status) = self.pending_statuses.get_mut(idx) else {
            return;
        };
        if let Status::Exhaustion(n) = status
            && *n > 1
        {
            *n -= 1;
            return;
        }
        self.pending_statuses.remove(idx);
        self.clamp_selected_status();
    }

    /// Pull `selected_status_index` back in bounds after `pending_statuses`
    /// shrinks — `None` if empty, otherwise at most the last entry. Mirrors
    /// `Encounter::clamp_indices`'s role for the main table's selection.
    fn clamp_selected_status(&mut self) {
        if self.pending_statuses.is_empty() {
            self.selected_status_index = None;
        } else if let Some(idx) = self.selected_status_index {
            self.selected_status_index = Some(idx.min(self.pending_statuses.len() - 1));
        }
    }

    pub fn clear(&mut self) {
        for field in self.fields_mut() {
            field.input.reset();
            field.valid = true;
        }
        self.amount = EditorInput {
            input: self.amount.input.clone().with_value("1".into()),
            kind: self.amount.kind,
            valid: self.amount.valid,
        };
        self.creature_type = CreatureType::default();
        self.hp_mode = HpMode::default();
        self.pending_statuses = Vec::new();
        self.selected_status_index = None;
        self.editing_index = None;
    }

    pub fn load_creature(&mut self, creature: &Creature, creature_index: Option<usize>) {
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
        self.initiative.input = self.initiative.input.clone().with_value(
            creature
                .get_initiative()
                .map(|n| n.to_string())
                .unwrap_or_default(),
        );
        self.hit_die.input = self
            .hit_die
            .input
            .clone()
            .with_value(creature.hit_die().unwrap_or("").to_string());
        self.creature_type = match creature {
            Creature::Player { .. } => CreatureType::Player,
            Creature::Monster { .. } => CreatureType::Monster,
        };
        // Not possible for an existing creature — see `toggle_hp_mode`.
        self.hp_mode = HpMode::Manual;
        self.pending_statuses = creature.get_statuses().clone();
        self.selected_status_index = if self.pending_statuses.is_empty() {
            None
        } else {
            Some(0)
        };
        for field in self.fields_mut() {
            field.valid = true;
        }
        self.editing_index = creature_index;
    }

    /// Validate every field against its [`FieldKind`], plus cross-field rules:
    /// current HP may not exceed max HP (Manual mode only — Roll mode
    /// computes both, so those fields don't participate), and Hit Die is
    /// required in Roll mode (optional otherwise). The Statuses box is a
    /// scratch/draft input — leftover un-submitted text never blocks the
    /// overall form. Sets each field's `valid` flag and returns `true` iff
    /// the form as a whole is submittable.
    pub fn validate_inputs(&mut self) -> bool {
        let mut all_valid = true;

        for field in self.fields_mut() {
            field.validate();
        }
        // The draft box's validity is judged at Enter-submit time, not here.
        self.statuses_input.valid = true;

        if !self.name.valid {
            all_valid = false;
        }
        if !self.ac.valid {
            all_valid = false;
        }
        if !self.cr.valid {
            all_valid = false;
        }
        if !self.amount.valid {
            all_valid = false;
        }
        if !self.initiative.valid {
            all_valid = false;
        }

        if self.hp_mode == HpMode::Manual {
            if !self.cur_hp.valid {
                all_valid = false;
            }
            if !self.max_hp.valid {
                all_valid = false;
            }
            if let (Ok(cur), Ok(max)) = (
                self.cur_hp.input.value().trim().parse::<u32>(),
                self.max_hp.input.value().trim().parse::<u32>(),
            ) && cur > max
            {
                self.cur_hp.valid = false;
                all_valid = false;
            }
        } else {
            // Roll mode: HP is computed from Hit Die, not typed — these
            // fields are hidden from the tab cycle, so stale/empty text in
            // them must not block or visibly invalidate submission.
            self.cur_hp.valid = true;
            self.max_hp.valid = true;
        }

        if self.hp_mode == HpMode::Roll {
            let hit_die_text = self.hit_die.input.value().trim();
            self.hit_die.valid = !hit_die_text.is_empty() && DiceExpr::parse(hit_die_text).is_ok();
            if !self.hit_die.valid {
                all_valid = false;
            }
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

    /// Build a single [`Creature`] from the current field values, using `name`
    /// in place of the Name field's own value (so numbered copies like
    /// "Goblin 1" can share one code path with a single submit). Honors
    /// `creature_type`: the `cr` field doubles as level (parsed as `u8`) for a
    /// [`Creature::Player`], or challenge rating (`f64`) for a `Monster`.
    /// In `HpMode::Roll`, rolls `hit_die` once and uses the total (min 1) for
    /// both max and current HP instead of the (hidden, unused) HP fields —
    /// `validate_inputs` already guarantees `hit_die` is non-empty and parses
    /// in that mode. Attaches `pending_statuses` to the built creature, and
    /// sets its initiative from the Initiative field if non-empty (leaving it
    /// unset, i.e. `None`, if the field was left blank or cleared).
    pub fn creature_with_name(&self, name: &str) -> Creature {
        let (_, field_max_hp, field_cur_hp, ac, cr) = self.parsed();
        let hit_die_text = self.hit_die.input.value().trim();
        let hit_die = if hit_die_text.is_empty() {
            None
        } else {
            Some(hit_die_text.to_string())
        };

        let (max_hp, cur_hp) = if self.hp_mode == HpMode::Roll {
            let rolled = hit_die
                .as_deref()
                .and_then(|expr| DiceExpr::parse(expr).ok())
                .and_then(|expr| expr.roll().ok())
                .map(|res| res.total.max(1) as u32)
                .unwrap_or(1);
            (rolled, rolled)
        } else {
            (field_max_hp, field_cur_hp)
        };

        let mut creature = match self.creature_type {
            CreatureType::Player => {
                Creature::new_player(name, max_hp, ac, Some(cur_hp), None, Some(cr as u8))
            }
            CreatureType::Monster => {
                Creature::new_monster(name, max_hp, ac, Some(cur_hp), None, Some(cr))
            }
        }
        .with_hit_die(hit_die);

        for status in &self.pending_statuses {
            creature.add_status(status.clone());
        }

        let initiative_text = self.initiative.input.value().trim();
        if let Ok(n) = initiative_text.parse::<u8>() {
            creature.set_initiative(n);
        }

        creature
    }

    /// Build a single [`Creature`] from the current field values, including
    /// the Name field.
    pub fn to_creature(&self) -> Creature {
        let name = self.name.input.value().trim().to_string();
        self.creature_with_name(&name)
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
    Initiative,
    HitDie,
    Statuses,
    Unfocused,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Creature;
    use crossterm::event::{KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::empty()))
    }

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

    fn type_into(e: &mut EditorState, field: EditorField, text: &str, encounter: &Encounter) {
        e.active_input = field;
        for c in text.chars() {
            e.handle_input_event(&key(KeyCode::Char(c)), encounter);
        }
    }

    // --- Hit Die field validation ---

    #[test]
    fn hit_die_field_accepts_empty_and_valid_expressions() {
        let mut field = EditorInput::new(FieldKind::DiceExpr);
        assert!(field.validate()); // empty is fine — hit_die is optional in Manual mode

        field.input = field.input.with_value("2d8+2".into());
        assert!(field.validate());
    }

    #[test]
    fn hit_die_field_rejects_garbage() {
        let mut field = EditorInput::new(FieldKind::DiceExpr);
        field.input = field.input.with_value("not dice".into());
        assert!(!field.validate());
    }

    // --- HP mode toggle ---

    #[test]
    fn toggle_hp_mode_flips_between_roll_and_manual() {
        let mut e = EditorState::default();
        assert_eq!(e.hp_mode, HpMode::Manual);
        e.toggle_hp_mode();
        assert_eq!(e.hp_mode, HpMode::Roll);
        e.toggle_hp_mode();
        assert_eq!(e.hp_mode, HpMode::Manual);
    }

    #[test]
    fn toggle_hp_mode_is_a_no_op_while_editing() {
        let mut e = EditorState::default();
        e.editing_index = Some(0);
        e.toggle_hp_mode();
        assert_eq!(e.hp_mode, HpMode::Manual);
    }

    #[test]
    fn load_creature_forces_manual_hp_mode_even_if_roll_was_active() {
        let mut e = EditorState::default();
        e.toggle_hp_mode(); // Roll, while not yet editing
        e.load_creature(
            &Creature::new_monster("Goblin", 7, 15, Some(7), None, Some(0.25)),
            Some(0),
        );
        assert_eq!(e.hp_mode, HpMode::Manual);
    }

    #[test]
    fn validate_inputs_requires_hit_die_only_in_roll_mode() {
        let mut e = valid_editor();
        assert!(e.validate_inputs()); // Manual mode, hit_die empty: fine

        e.toggle_hp_mode(); // Roll
        assert!(!e.validate_inputs()); // hit_die now required

        e.hit_die.input = e.hit_die.input.clone().with_value("2d8+2".into());
        assert!(e.validate_inputs());
    }

    #[test]
    fn roll_mode_creature_uses_hit_die_total_for_both_hp_values() {
        let mut e = valid_editor();
        e.toggle_hp_mode();
        // A fixed, non-flaky expression: 1d1 always rolls 1, +5 makes 6.
        e.hit_die.input = e.hit_die.input.clone().with_value("1d1+5".into());
        assert!(e.validate_inputs());

        let creature = e.to_creature();
        assert_eq!(creature.max_hp(), 6);
        assert_eq!(creature.hp(), 6);
        assert_eq!(creature.hit_die(), Some("1d1+5"));
    }

    #[test]
    fn manual_mode_creature_uses_the_hp_fields_and_optional_hit_die() {
        let creature = valid_editor().to_creature();
        assert_eq!(creature.max_hp(), 7);
        assert_eq!(creature.hp(), 7);
        assert_eq!(creature.hit_die(), None);
    }

    // --- Status add/parse ---

    #[test]
    fn try_submit_status_adds_a_known_status() {
        let mut e = EditorState::default();
        let encounter = Encounter::default();
        e.statuses_input.input = e.statuses_input.input.clone().with_value("Poisoned".into());

        assert!(e.try_submit_status(&encounter));
        assert_eq!(e.pending_statuses, vec![Status::Poisoned]);
        assert_eq!(e.statuses_input.input.value(), ""); // box cleared
        assert_eq!(e.selected_status_index, Some(0));
    }

    #[test]
    fn try_submit_status_is_case_insensitive() {
        let mut e = EditorState::default();
        let encounter = Encounter::default();
        e.statuses_input.input = e.statuses_input.input.clone().with_value("pOiSoNeD".into());

        assert!(e.try_submit_status(&encounter));
        assert_eq!(e.pending_statuses, vec![Status::Poisoned]);
    }

    #[test]
    fn try_submit_status_rejects_unknown_word() {
        let mut e = EditorState::default();
        let encounter = Encounter::default();
        e.statuses_input.input = e.statuses_input.input.clone().with_value("Confused".into());

        assert!(!e.try_submit_status(&encounter));
        assert!(e.pending_statuses.is_empty());
        assert!(!e.statuses_input.valid);
        assert_eq!(e.statuses_input.input.value(), "Confused"); // left as-is
    }

    #[test]
    fn try_submit_status_on_empty_box_does_nothing() {
        let mut e = EditorState::default();
        let encounter = Encounter::default();
        assert!(!e.try_submit_status(&encounter));
        assert!(e.pending_statuses.is_empty());
    }

    #[test]
    fn try_submit_status_bumps_exhaustion_count_instead_of_duplicating() {
        let mut e = EditorState::default();
        let encounter = Encounter::default();

        e.statuses_input.input = e
            .statuses_input
            .input
            .clone()
            .with_value("Exhaustion".into());
        e.try_submit_status(&encounter);
        e.statuses_input.input = e
            .statuses_input
            .input
            .clone()
            .with_value("Exhaustion".into());
        e.try_submit_status(&encounter);

        assert_eq!(e.pending_statuses, vec![Status::Exhaustion(2)]);
    }

    #[test]
    fn try_submit_status_resolves_grappled_target_by_name() {
        let mut e = EditorState::default();
        let mut encounter = Encounter::default();
        let grappler = Creature::new_monster("Ogre", 30, 12, None, None, Some(2.0));
        let grappler_id = grappler.id();
        encounter.add_creature(grappler);
        e.statuses_input.input = e
            .statuses_input
            .input
            .clone()
            .with_value("Grappled Ogre".into());

        assert!(e.try_submit_status(&encounter));
        assert_eq!(e.pending_statuses, vec![Status::Grappled(grappler_id)]);
    }

    #[test]
    fn try_submit_status_rejects_grappled_with_unresolvable_target() {
        let mut e = EditorState::default();
        let encounter = Encounter::default(); // no creatures to match
        e.statuses_input.input = e
            .statuses_input
            .input
            .clone()
            .with_value("Grappled Nobody".into());

        assert!(!e.try_submit_status(&encounter));
        assert!(e.pending_statuses.is_empty());
    }

    // --- Status delete/select ---

    #[test]
    fn delete_selected_status_removes_a_plain_status() {
        let mut e = EditorState::default();
        e.pending_statuses = vec![Status::Poisoned, Status::Prone];
        e.selected_status_index = Some(0);

        e.delete_selected_status();

        assert_eq!(e.pending_statuses, vec![Status::Prone]);
        assert_eq!(e.selected_status_index, Some(0));
    }

    #[test]
    fn delete_selected_status_decrements_exhaustion_then_removes_at_zero() {
        let mut e = EditorState::default();
        e.pending_statuses = vec![Status::Exhaustion(2)];
        e.selected_status_index = Some(0);

        e.delete_selected_status();
        assert_eq!(e.pending_statuses, vec![Status::Exhaustion(1)]);

        e.delete_selected_status();
        assert!(e.pending_statuses.is_empty());
        assert_eq!(e.selected_status_index, None);
    }

    #[test]
    fn delete_selected_status_clamps_selection_after_shrinking() {
        let mut e = EditorState::default();
        e.pending_statuses = vec![Status::Poisoned, Status::Prone, Status::Blinded];
        e.selected_status_index = Some(2); // last entry

        e.delete_selected_status();

        assert_eq!(e.pending_statuses.len(), 2);
        assert_eq!(e.selected_status_index, Some(1)); // pulled back in bounds
    }

    #[test]
    fn backspace_deletes_selected_status_only_when_box_is_empty() {
        let mut e = EditorState::default();
        let encounter = Encounter::default();
        e.pending_statuses = vec![Status::Poisoned];
        e.selected_status_index = Some(0);
        e.active_input = EditorField::Statuses;

        // Box has text: Backspace edits the text, not the status list.
        type_into(&mut e, EditorField::Statuses, "x", &encounter);
        e.handle_input_event(&key(KeyCode::Backspace), &encounter);
        assert_eq!(e.pending_statuses.len(), 1); // untouched
        assert_eq!(e.statuses_input.input.value(), "");

        // Box is empty: Backspace now deletes the selected status.
        e.handle_input_event(&key(KeyCode::Backspace), &encounter);
        assert!(e.pending_statuses.is_empty());
    }

    #[test]
    fn left_right_navigate_selection_only_when_box_is_empty() {
        let mut e = EditorState::default();
        let encounter = Encounter::default();
        e.pending_statuses = vec![Status::Poisoned, Status::Prone];
        e.selected_status_index = Some(0);
        e.active_input = EditorField::Statuses;

        e.handle_input_event(&key(KeyCode::Right), &encounter);
        assert_eq!(e.selected_status_index, Some(1));

        e.handle_input_event(&key(KeyCode::Left), &encounter);
        assert_eq!(e.selected_status_index, Some(0));
    }

    // --- Tab cycle ---

    #[test]
    fn tab_cycle_skips_hp_fields_in_roll_mode() {
        let mut e = EditorState::default();
        e.toggle_hp_mode(); // Roll
        e.active_input = EditorField::Name;
        e.next_field();
        assert_eq!(e.active_input, EditorField::AC); // CurrentHP/MaxHP skipped
    }

    #[test]
    fn tab_cycle_includes_hp_fields_in_manual_mode() {
        let mut e = EditorState::default();
        e.active_input = EditorField::Name;
        e.next_field();
        assert_eq!(e.active_input, EditorField::CurrentHP);
    }

    #[test]
    fn tab_cycle_reaches_hit_die_and_statuses_and_wraps() {
        let mut e = EditorState::default();
        e.active_input = EditorField::Amount;
        e.next_field();
        assert_eq!(e.active_input, EditorField::Initiative);
        e.next_field();
        assert_eq!(e.active_input, EditorField::HitDie);
        e.next_field();
        assert_eq!(e.active_input, EditorField::Statuses);
        e.next_field();
        assert_eq!(e.active_input, EditorField::Name); // wraps
    }

    // --- Initiative field ---

    #[test]
    fn initiative_field_accepts_empty_and_valid_values() {
        let mut field = EditorInput::new(FieldKind::OptionalUint { min: 0, max: 99 });
        assert!(field.validate()); // empty is fine — not every creature has rolled

        field.input = field.input.with_value("15".into());
        assert!(field.validate());
    }

    #[test]
    fn initiative_field_rejects_out_of_range_and_non_numeric() {
        let mut field = EditorInput::new(FieldKind::OptionalUint { min: 0, max: 99 });
        field.input = field.input.with_value("100".into());
        assert!(!field.validate());

        field.input = field.input.with_value("nope".into());
        assert!(!field.validate());
    }

    #[test]
    fn creature_with_name_sets_initiative_when_provided() {
        let mut e = valid_editor();
        e.initiative.input = e.initiative.input.clone().with_value("15".into());

        let creature = e.to_creature();
        assert_eq!(creature.get_initiative(), Some(15));
    }

    #[test]
    fn creature_with_name_leaves_initiative_unset_when_field_is_empty() {
        let creature = valid_editor().to_creature();
        assert_eq!(creature.get_initiative(), None);
    }

    #[test]
    fn load_creature_populates_initiative_and_clearing_it_unsets_on_submit() {
        let mut e = EditorState::default();
        let mut goblin = Creature::new_monster("Goblin", 7, 15, Some(7), None, Some(0.25));
        goblin.set_initiative(11);
        e.load_creature(&goblin, Some(0));
        assert_eq!(e.initiative.input.value(), "11");

        // Round-trips untouched.
        assert_eq!(e.to_creature().get_initiative(), Some(11));

        // Clearing the field and submitting unsets it.
        e.initiative.input = e.initiative.input.clone().with_value("".into());
        assert_eq!(e.to_creature().get_initiative(), None);
    }
}
