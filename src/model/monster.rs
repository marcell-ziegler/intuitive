use dice_parser::{DiceExpr, RollSpec};

use crate::model::{
    creature::{Creature, CreatureProperties, DamageOutcome},
    stats::Stats,
    status::Status,
};

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
