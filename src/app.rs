use crate::model::Creature;

pub struct App {
    pub players: Vec<Creature>,
}

impl Default for App {
    fn default() -> Self {
        App {
            players: Vec::new(),
        }
    }
}
