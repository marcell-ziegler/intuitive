use intuitive::model::Player;

pub struct App {
    pub players: Vec<Player>,
}

impl Default for App {
    fn default() -> Self {
        App {
            players: Vec::new(),
        }
    }
}
