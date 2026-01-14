use uuid::Uuid;

pub struct App {}

impl Default for App {
    fn default() -> Self {
        App {}
    }
}

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
}

#[derive(Debug, PartialEq)]
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
    Unconcious,
}

/// Common properties of types that are Creature
///
/// * `name`: A player name, or statblock name, for the Creature
/// * `hp`: Current health of the Creature
/// * `max_hp`: Maximum health of the Creature
/// * `ac`: Armor Class of creature
/// * `is_dead`: wether the Creature is dead.
/// * `statuses`: `Vec<Status>` of all statuses currently affecting the Creature.
struct CreatureProperties {
    name: String,
    hp: u32,
    max_hp: u32,
    ac: u32,
    is_dead: bool,
    statuses: Vec<Status>,
    initiative: Option<u8>,
}

pub struct Player {
    props: CreatureProperties,
    is_down: bool,
}

impl Player {
    fn new(name: &str, max_hp: u32, ac: u32) -> Self {
        Player {
            props: CreatureProperties {
                name: String::from(name),
                max_hp,
                hp: max_hp,
                ac,
                is_dead: false,
                statuses: Vec::new(),
                initiative: None,
            },
            is_down: false,
        }
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
        self.props.hp = (self.props.hp + amount).max(self.max_hp())
    }

    fn damage(&mut self, amount: u32) -> DamageOutcome {
        let curr_hp = self.props.hp as i32;
        let delta = curr_hp - (amount as i32);

        // Set hp to 0 or whatever it is after the damage.
        self.props.hp = (curr_hp - amount as i32).max(0) as u32;

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
        self.props.statuses.push(status);
    }

    fn clear_status(&mut self) {
        self.props.statuses.clear();
    }

    fn get_statuses(&self) -> &Vec<Status> {
        &self.props.statuses
    }
}
