use dice_parser::{DiceExpr, RollSpec};

use crate::model::{
    creature::{Creature, CreatureProperties, CreatureTrait, DamageOutcome},
    stats::Stats,
    status::Status,
};

#[derive(Debug, Clone)]
pub struct Monster {
    pub props: CreatureProperties,
    pub cr: f64,
}

impl Monster {
    /// Make an instance of `Monster`
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
        cr: Option<f64>,
    ) -> Self {
        Monster {
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
            cr: cr.unwrap_or(0.0),
        }
    }
}
