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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    pub strength: u8,
    pub dexterity: u8,
    pub constitution: u8,
    pub intelligence: u8,
    pub wisdom: u8,
    pub charisma: u8,
}
impl Stats {
    /// Create a new `Stats` with values clamped to 0 <= x <= 30
    ///
    /// * `str`: Strength
    /// * `dex`: Dexterity
    /// * `con`: Constitution
    /// * `int`: Intelligence
    /// * `wis`: Wisdom
    /// * `cha`: Charisma
    pub fn new(str: u8, dex: u8, con: u8, int: u8, wis: u8, cha: u8) -> Self {
        let strength = str.min(30);
        let dexterity = dex.min(30);
        let constitution = con.min(30);
        let intelligence = int.min(30);
        let wisdom = wis.min(30);
        let charisma = cha.min(30);

        Stats {
            strength,
            dexterity,
            constitution,
            intelligence,
            wisdom,
            charisma,
        }
    }

    fn to_mod(stat: u8) -> i8 {
        if stat == 1 {
            -5
        } else if stat <= 3 {
            -4
        } else if stat <= 5 {
            -3
        } else if stat <= 7 {
            -2
        } else if stat <= 9 {
            -1
        } else if stat <= 11 {
            0
        } else if stat <= 13 {
            1
        } else if stat <= 15 {
            2
        } else if stat <= 17 {
            3
        } else if stat <= 19 {
            4
        } else if stat <= 21 {
            5
        } else if stat <= 23 {
            6
        } else if stat <= 25 {
            7
        } else if stat <= 27 {
            8
        } else if stat <= 29 {
            9
        } else {
            10
        }
    }

    /// Return the strength modifier
    pub fn str_mod(&self) -> i8 {
        Stats::to_mod(self.strength)
    }

    /// Return the dexterity modifier
    pub fn dex_mod(&self) -> i8 {
        Stats::to_mod(self.dexterity)
    }
    /// Return the constitution modifier
    pub fn con_mod(&self) -> i8 {
        Stats::to_mod(self.constitution)
    }
    /// Return the intelligence modifier
    pub fn int_mod(&self) -> i8 {
        Stats::to_mod(self.intelligence)
    }
    /// return the wisdom modifier
    pub fn wis_mod(&self) -> i8 {
        Stats::to_mod(self.wisdom)
    }
    /// Return the charisma modifier
    pub fn cha_mod(&self) -> i8 {
        Stats::to_mod(self.charisma)
    }
}
impl Default for Stats {
    fn default() -> Self {
        Stats {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        }
    }
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
    stats: Stats,
}

#[derive(Debug, Clone)]
pub struct Player {
    props: CreatureProperties,
}

impl Player {
    /// Make an instance of `Player`
    ///
    /// * `name`: The `Player`'s name
    /// * `max_hp`: The `Player`'s max hp
    /// * `ac`: The `Player`'s armour class
    /// * `current_hp`: an `Option<u32>` containing None if cur_hp is max_hp otherwise the current hp. An invalid current hp automatically becomes max_hp.
    pub fn new(
        name: &str,
        max_hp: u32,
        ac: u32,
        current_hp: Option<u32>,
        stats: Option<Stats>,
    ) -> Self {
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
                stats: stats.unwrap_or_default(),
            },
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
        if self.hp() == 0 && amount > 0 {
            self.props.is_dead = false;
        }
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
            self.props.is_dead = true;
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
        self.props.initiative = Some(value);
    }

    fn roll_initiative(&mut self) -> u8 {
        let initiative = (rand::random_range(1..=20) + self.stats().dex_mod()).max(0) as u8;
        self.props.initiative = Some(initiative);
        initiative
    }

    fn clear_initative(&mut self) {
        self.props.initiative = None;
    }

    fn stats(&self) -> &Stats {
        &self.props.stats
    }
}

#[derive(Debug, Clone)]
pub struct Monster {
    props: CreatureProperties,
    cr: f64,
}

impl Creature for Monster {
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
        if self.hp() == 0 && amount > 0 {
            self.props.is_dead = false;
        }
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
            self.props.is_dead = true;
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
        self.props.initiative = Some(value);
    }

    fn roll_initiative(&mut self) -> u8 {
        let initiative = (rand::random_range(1..=20) + self.stats().dex_mod()).max(0) as u8;
        self.props.initiative = Some(initiative);
        initiative
    }

    fn clear_initative(&mut self) {
        self.props.initiative = None;
    }

    fn stats(&self) -> &Stats {
        &self.props.stats
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_player_new_and_getters() {
        let player = Player::new("Alice", 30, 15, None, None);
        assert_eq!(player.name(), "Alice");
        assert_eq!(player.max_hp(), 30);
        assert_eq!(player.hp(), 30);
        assert_eq!(player.ac(), 15);
        assert_eq!(*player.stats(), Stats::default());
        assert!(player.is_alive());
        assert!(!player.is_dead());
    }

    #[test]
    fn test_player_new_with_current_hp() {
        let player = Player::new("Bob", 40, 12, Some(25), None);
        assert_eq!(player.hp(), 25);

        let player = Player::new("Carol", 40, 12, Some(50), None);
        assert_eq!(player.hp(), 40); // invalid current_hp defaults to max_hp
    }

    #[test]
    fn test_heal_and_damage() {
        let mut player = Player::new("Dave", 20, 10, Some(10), None);
        player.heal(5);
        assert_eq!(player.hp(), 15);

        player.heal(10);
        assert_eq!(player.hp(), 20); // should not exceed max_hp

        let outcome = player.damage(5);
        assert_eq!(outcome, DamageOutcome::Survived);
        assert_eq!(player.hp(), 15);

        let outcome = player.damage(30);
        assert_eq!(outcome, DamageOutcome::Downed);
        assert_eq!(player.hp(), 0);

        let outcome = player.damage(20);
        assert_eq!(outcome, DamageOutcome::Died);
        assert!(player.is_dead());
    }

    #[test]
    fn test_statuses() {
        let mut player = Player::new("Eve", 10, 10, None, None);
        player.add_status(Status::Blinded);
        assert!(player.get_statuses().contains(&Status::Blinded));

        player.add_status(Status::Blinded);
        assert_eq!(
            player
                .get_statuses()
                .iter()
                .filter(|s| **s == Status::Blinded)
                .count(),
            1
        );

        player.add_status(Status::Poisoned);
        assert!(player.get_statuses().contains(&Status::Poisoned));

        assert!(player.remove_status(Status::Blinded).is_some());
        assert!(!player.get_statuses().contains(&Status::Blinded));

        player.clear_status();
        assert!(player.get_statuses().is_empty());
    }

    #[test]
    fn test_initiative() {
        let mut player = Player::new("Frank", 10, 10, None, None);
        let roll = player.roll_initiative();
        assert!((1..=20).contains(&roll));
        assert_eq!(player.get_initiative(), roll);

        player.set_initiative(15);
        assert_eq!(player.get_initiative(), 15);

        player.clear_initative();
        let new_roll = player.get_initiative();
        assert!((1..=20).contains(&new_roll));
    }
}
