use serde::{Deserialize, Serialize};

use crate::model::Creature;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encounter {
    pub name: String,
    pub creatures: Vec<Creature>,
    pub initiative_index: usize,
    pub cursor_index: usize,
}

impl Default for Encounter {
    fn default() -> Self {
        Encounter {
            name: String::new(),
            creatures: vec![],
            initiative_index: 0,
            cursor_index: 0,
        }
    }
}
