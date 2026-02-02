use dice_parser::{DiceExpr, RollSpec};

use crate::model::{
    creature::{Creature, CreatureProperties, DamageOutcome},
    stats::Stats,
    status::Status,
};

#[derive(Debug, Clone)]
pub(crate) struct Player {
    props: CreatureProperties,
    level: u8,
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
        level: Option<u8>,
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
            level: level.unwrap_or(1),
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
        let expr = DiceExpr::Sum(
            // 1d20
            Box::new(DiceExpr::Roll(RollSpec::new(1, 20, None))),
            // + Dex modifier
            Box::new(DiceExpr::Literal(self.stats().dex_mod().into())),
        );
        expr.roll().unwrap().total as u8
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
        let player = Player::new("Alice", 30, 15, None, None, None);
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
        let player = Player::new("Bob", 40, 12, Some(25), None, None);
        assert_eq!(player.hp(), 25);

        let player = Player::new("Carol", 40, 12, Some(50), None, None);
        assert_eq!(player.hp(), 40); // invalid current_hp defaults to max_hp
    }

    #[test]
    fn test_heal_and_damage() {
        let mut player = Player::new("Dave", 20, 10, Some(10), None, None);
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
        let mut player = Player::new("Eve", 10, 10, None, None, None);
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
        let mut player = Player::new("Frank", 10, 10, None, None, None);
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
