use uuid::Uuid;

use crate::model::{stats::Stats, status::Status};

pub type CreatureId = Uuid;

#[derive(Debug, PartialEq, Eq)]
pub enum DamageOutcome {
    Survived,
    Downed,
    Died,
}

/// Common getters for creatures
pub trait Creature {
    /// Return the current health of the Creature
    fn hp(&self) -> u32;

    /// Return the current health of the Creature
    fn max_hp(&self) -> u32;

    /// Return the creatures armor class
    fn ac(&self) -> u32;

    /// Return the name of the creature
    fn name(&self) -> &str;

    /// Add `amount` to the creatures `hp` up to `max_hp`.
    fn heal(&mut self, amount: u32);

    /// Lower the creatures `hp` by `amount`.
    ///
    /// # Returns
    /// If the creature is a player and `hp - amount >= -max_hp` then returns
    /// `DamageOutcome::Downed`. If `hp - amount <= -max_hp` returns `DamageOutcome::Died`.
    ///
    /// If not a player, returns `DamageOutcome::Died` if `hp - amount <= 0`
    ///
    /// Always returns `DamageOutcome::Survived` if `hp - amount > 0`.
    fn damage(&mut self, amount: u32) -> DamageOutcome;

    /// Returns `true` if Creature is dead.
    fn is_dead(&self) -> bool;

    /// Returns `true` if Craeture is alive (not dead).
    fn is_alive(&self) -> bool;

    /// Borrow teh status vector in a Creature
    fn get_statuses(&self) -> &Vec<Status>;

    /// Add `status` to the end of the vector of Statuses.
    ///
    /// * `status`: the `Status` to be added.
    fn add_status(&mut self, status: Status);

    /// Remove the given `Status` from the Creature
    ///
    /// * `status`: The status to be removed
    ///
    /// # Returns
    /// Some(()) if success, otherwise None.
    fn remove_status(&mut self, status: Status) -> Option<()>;

    /// Remove all Statuses from the Creature
    fn clear_status(&mut self);

    /// Set the creatures initiative to a random u8 between 0..=20 + the players initiative (Dex) modifier and return the value
    fn roll_initiative(&mut self) -> u8;

    /// Set the players initiative
    fn set_initiative(&mut self, value: u8);

    /// Clear the initiative of the Creature, i.e. set it to None
    fn clear_initative(&mut self);

    /// Get the creatures initiative, or if it is `None``: roll it and return the new value.
    fn get_initiative(&mut self) -> u8;

    /// Return an immutable borrow if the Creatures stats
    fn stats(&self) -> &Stats;
}

/// Common properties of types that are Creature
///
/// * `name`: A player name, or statblock name, for the Creature
/// * `hp`: Current health of the Creature
/// * `max_hp`: Maximum health of the Creature
/// * `ac`: Armor Class of creature
/// * `is_dead`: wether the Creature is dead.
/// * `statuses`: `Vec<Status>` of all statuses currently affecting the Creature.
#[derive(Debug, Clone)]
pub struct CreatureProperties {
    pub name: String,
    pub hp: u32,
    pub max_hp: u32,
    pub ac: u32,
    pub is_dead: bool,
    pub statuses: Vec<Status>,
    pub initiative: Option<u8>,
    pub stats: Stats,
}
