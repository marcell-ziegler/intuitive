//! [TODO:description]

use rand;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq)]
pub enum DamageOutcome {
    Survived,
    Downed,
    Died,
}

pub type CreatureId = Uuid;

/// Common getters for creatures
/// [TODO:description]
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

    /// Set the creatures initiative to a random u8 between 0..=20 and return the value
    fn roll_initiative(&mut self) -> u8;

    /// Set the players initiative to a 1 <= u8 <= 20
    ///
    /// # Panics
    /// Panics if `value` is not between 1-20
    fn set_initiative(&mut self, value: u8);

    /// Clear the initiative of the Creature, i.e. set it to None
    fn clear_initative(&mut self);

    /// Get the creatures initiative, or if it is `None``: roll it and return the new value.
    fn get_initiative(&mut self) -> u8;
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Status {
    Blinded,
    Charmed,
    Deafened,
    Exhaustion(u8),
    Frightened,
    Grappled(CreatureId),
    Incapacitated,
    Invisible,
    Paralyzed,
    Petrified,
    Poisoned,
    Prone,
    Restrained,
    Stunned,
    Unconscious,
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
struct CreatureProperties {
    name: String,
    hp: u32,
    max_hp: u32,
    ac: u32,
    is_dead: bool,
    statuses: Vec<Status>,
    initiative: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct Player {
    props: CreatureProperties,
    is_down: bool,
}

impl Player {
    /// Make an instance of `Player`
    ///
    /// * `name`: The `Player`'s name
    /// * `max_hp`: The `Player`'s max hp
    /// * `ac`: The `Player`'s armour class
    /// * `current_hp`: an `Option<u32>` containing None if cur_hp is max_hp otherwise the current hp. An invalid current hp automatically becomes max_hp.
    pub fn new(name: &str, max_hp: u32, ac: u32, current_hp: Option<u32>) -> Self {
        Player {
            props: CreatureProperties {
                name: String::from(name),
                max_hp,
                hp: if let Some(chp) = current_hp
                    && chp <= max_hp
                {
                    chp
                } else {
                    max_hp
                },
                ac,
                is_dead: false,
                statuses: Vec::new(),
                initiative: None,
            },
            is_down: false,
        }
    }

    /// Return wether the `Player` is down
    pub fn is_down(&self) -> bool {
        self.is_down
    }
}

impl Creature for Player {
    fn hp(&self) -> u32 {
        self.props.hp
    }
    fn ac(&self) -> u32 {
        self.props.ac
    }

    fn name(&self) -> &str {
        &self.props.name[..]
    }

    fn max_hp(&self) -> u32 {
        self.props.max_hp
    }

    fn is_dead(&self) -> bool {
        self.props.is_dead
    }

    fn is_alive(&self) -> bool {
        !self.props.is_dead
    }

    fn heal(&mut self, amount: u32) {
        self.props.hp = (self.props.hp + amount).min(self.max_hp())
    }

    fn damage(&mut self, amount: u32) -> DamageOutcome {
        let curr_hp = self.props.hp as i32;
        let delta = curr_hp - (amount as i32);

        // Set hp to 0 or whatever it is after the damage.
        self.props.hp = delta.max(0) as u32;

        if delta > 0 {
            DamageOutcome::Survived
        } else if delta <= -(self.max_hp() as i32) {
            DamageOutcome::Died
        } else {
            DamageOutcome::Downed
        }
    }

    fn remove_status(&mut self, status: Status) -> Option<()> {
        if let Some(i) = self.props.statuses.iter().position(|x| x == &status) {
            self.props.statuses.remove(i);
            Some(())
        } else {
            None
        }
    }

    fn add_status(&mut self, status: Status) {
        if !self.props.statuses.contains(&status) {
            self.props.statuses.push(status);
        }
    }

    fn clear_status(&mut self) {
        self.props.statuses.clear();
    }

    fn get_statuses(&self) -> &Vec<Status> {
        &self.props.statuses
    }

    fn get_initiative(&mut self) -> u8 {
        if let Some(initiative) = self.props.initiative {
            initiative
        } else {
            self.roll_initiative()
        }
    }

    fn set_initiative(&mut self, value: u8) {
        if !(1..=20).contains(&value) {
            panic!("Invalid initiative input: {value}")
        }
        self.props.initiative = Some(value);
    }

    fn roll_initiative(&mut self) -> u8 {
        let initiative = rand::random_range(1..=20);
        self.props.initiative = Some(initiative);
        initiative
    }

    fn clear_initative(&mut self) {
        self.props.initiative = None;
    }
}

#[derive(Debug, Clone)]
pub struct Monster {
    props: CreatureProperties,
    cr: f64,
}
