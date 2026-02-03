use dice_parser::{DiceExpr, RollSpec};
use uuid::Uuid;

use crate::model::{stats::Stats, status::Status};

pub type CreatureId = Uuid;

#[derive(Debug, PartialEq, Eq)]
pub enum DamageOutcome {
    Survived,
    Downed,
    Died,
}

pub enum Creature {
    Player {
        props: CreatureProperties,
        level: u8,
    },
    Monster {
        props: CreatureProperties,
        cr: f64,
    },
}

impl Creature {
    pub fn new_player(
        name: &str,
        max_hp: u32,
        ac: u32,
        current_hp: Option<u32>,
        stats: Option<Stats>,
        level: Option<u8>,
    ) -> Self {
        Creature::Player {
            props: CreatureProperties::new(
                name.to_string(),
                current_hp.unwrap_or(max_hp),
                max_hp,
                ac,
                stats.unwrap_or_default(),
            ),
            level: level.unwrap_or(1),
        }
    }
    pub fn new_monster(
        name: &str,
        max_hp: u32,
        ac: u32,
        current_hp: Option<u32>,
        stats: Option<Stats>,
        cr: Option<f64>,
    ) -> Self {
        Creature::Monster {
            props: CreatureProperties::new(
                name.to_string(),
                current_hp.unwrap_or(max_hp),
                max_hp,
                ac,
                stats.unwrap_or_default(),
            ),
            cr: cr.unwrap_or(0.0),
        }
    }

    pub fn props(&self) -> &CreatureProperties {
        match self {
            Creature::Player { props: p, level: _ } => p,
            Creature::Monster { props: p, cr: _ } => p,
        }
    }

    fn props_mut(&mut self) -> &mut CreatureProperties {
        match self {
            Creature::Player {
                props: p_mut,
                level: _,
            } => p_mut,
            Creature::Monster {
                props: p_mut,
                cr: _,
            } => p_mut,
        }
    }

    /// Return the current health of the Creature
    pub fn hp(&self) -> u32 {
        self.props().hp
    }

    fn hp_mut(&mut self) -> &mut u32 {
        &mut self.props_mut().hp
    }

    /// Return the current health of the Creature
    pub fn max_hp(&self) -> u32 {
        self.props().max_hp
    }

    /// Return the creatures armor class
    pub fn ac(&self) -> u32 {
        self.props().ac
    }

    /// Return the name of the creature
    pub fn name(&self) -> &str {
        &self.props().name[..]
    }

    /// Returns the stats of the `Creature` as `Stats`
    pub fn stats(&self) -> Stats {
        self.props().stats
    }

    /// Returns `true` if `Creature` is dead.
    pub fn is_dead(&self) -> bool {
        self.props().is_dead
    }

    fn is_dead_mut(&mut self) -> &mut bool {
        &mut self.props_mut().is_dead
    }

    /// Returns `true` if `Creature` is alive (not dead).
    pub fn is_alive(&self) -> bool {
        !self.props().is_dead
    }

    fn initiative_mut(&mut self) -> &mut Option<u8> {
        &mut self.props_mut().initiative
    }

    /// Add `amount` to the creatures `hp` up to `max_hp`.
    pub fn heal(&mut self, amount: u32) {
        if self.hp() == 0 && amount > 0 {
            *self.is_dead_mut() = false
        }
        *self.hp_mut() = (self.hp() + amount).min(self.max_hp());
    }

    /// Lower the creatures `hp` by `amount`.
    ///
    /// # Returns
    /// If the creature is a player and `hp - amount >= -max_hp` then returns
    /// `DamageOutcome::Downed`. If `hp - amount <= -max_hp` returns `DamageOutcome::Died`.
    ///
    /// If not a player, returns `DamageOutcome::Died` if `hp - amount <= 0`
    ///
    /// Always returns `DamageOutcome::Survived` if `hp - amount > 0`.
    pub fn damage(&mut self, amount: u32) -> DamageOutcome {
        let curr_hp = self.hp() as i32;
        let delta = curr_hp - (amount as i32);

        // Set hp to 0 or whatever it is after the damage.
        *self.hp_mut() = delta.max(0) as u32;

        if delta > 0 {
            DamageOutcome::Survived
        } else if delta <= -(self.max_hp() as i32) {
            *self.is_dead_mut() = true;
            DamageOutcome::Died
        } else {
            DamageOutcome::Downed
        }
    }

    /// Borrow the status vector in a `Creature` variant
    pub fn get_statuses(&self) -> &Vec<Status> {
        &self.props().statuses
    }

    /// Add `status` to the end of the vector of Statuses.
    ///
    /// * `status`: the `Status` to be added.
    pub fn add_status(&mut self, status: Status) {
        if !self.props().statuses.contains(&status) {
            self.props_mut().statuses.push(status);
        }
    }
    ///
    /// Remove the given `Status` from the Creature. If the status does not exist for the Creature, nothing is done.
    ///
    /// * `status`: The status to be removed
    ///
    ///
    pub fn remove_status(&mut self, status: Status) {
        if let Some(i) = self.props().statuses.iter().position(|x| x == &status) {
            self.props_mut().statuses.remove(i);
        }
    }

    /// Remove all Statuses from the Creature
    pub fn clear_status(&mut self) {
        self.props_mut().statuses.clear();
    }

    /// Sets the `Creature`'s initiative to a random value rolled as 1d20 + `self.dex_mod()` is in
    /// the D&D rules.
    pub fn roll_initiative(&mut self) -> u8 {
        let expr = DiceExpr::Sum(
            // 1d20
            Box::new(DiceExpr::Roll(RollSpec::new(1, 20, None))),
            // + Dex modifier
            Box::new(DiceExpr::Literal(self.stats().dex_mod().into())),
        );
        let res = expr.roll().unwrap().total as u8;
        *self.initiative_mut() = Some(res);
        res
    }

    /// Set the `Creature`'s initiative to the specified `value`.
    ///
    /// * `value`: initiative value to set
    pub fn set_initiative(&mut self, value: u8) {
        self.props_mut().initiative = Some(value)
    }

    /// Set the initiative to `None`
    pub fn clear_initative(&mut self) {
        *self.initiative_mut() = None;
    }

    /// Gets the current initiative, or if it is `None` rolls a new initiative
    pub fn get_initiative(&mut self) -> u8 {
        if let Some(i) = self.props().initiative {
            i
        } else {
            self.roll_initiative()
        }
    }

    /// `true` if initiative is not `None`
    pub fn has_initative(&self) -> bool {
        self.props().initiative.is_some()
    }
}

/// Common properties of `Creature` variants
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

impl CreatureProperties {
    pub fn new(name: String, cur_hp: u32, max_hp: u32, ac: u32, stats: Stats) -> Self {
        CreatureProperties {
            name,
            hp: cur_hp.min(max_hp),
            max_hp,
            ac,
            is_dead: cur_hp == 0,
            initiative: None,
            statuses: Vec::new(),
            stats,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::model::Creature;

    use super::*;
    #[test]
    fn test_player_new_and_getters() {
        let player = Creature::new_player("Alice", 30, 15, None, None, None);
        assert_eq!(player.name(), "Alice");
        assert_eq!(player.max_hp(), 30);
        assert_eq!(player.hp(), 30);
        assert_eq!(player.ac(), 15);
        assert_eq!(player.stats(), Stats::default());
        assert!(player.is_alive());
        assert!(!player.is_dead());
    }

    #[test]
    fn test_player_new_with_current_hp() {
        let player = Creature::new_player("Bob", 40, 12, Some(25), None, None);
        assert_eq!(player.hp(), 25);

        let player = Creature::new_player("Carol", 40, 12, Some(50), None, None);
        assert_eq!(player.hp(), 40); // invalid current_hp defaults to max_hp
    }

    #[test]
    fn test_heal_and_damage() {
        let mut player = Creature::new_player("Dave", 20, 10, Some(10), None, None);
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
        let mut player = Creature::new_player("Eve", 10, 10, None, None, None);
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

        player.remove_status(Status::Blinded);
        assert!(!player.get_statuses().contains(&Status::Blinded));

        player.clear_status();
        assert!(player.get_statuses().is_empty());
    }

    #[test]
    fn test_initiative() {
        let mut player = Creature::new_player("Frank", 10, 10, None, None, None);
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
