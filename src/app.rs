use ratatui::widgets::ListState;

use crate::model::Creature;

#[derive(Debug)]
pub struct App {
    creatures: Vec<Creature>,
    pub current_screen: CurrentScreen,
    pub creature_list_state: ListState,
}

impl App {
    pub fn creature_representations(&self) -> Vec<String> {
        let mut reprs: Vec<String> = Vec::new();

        for creature in &self.creatures {
            reprs.push(format!("{:?}", creature));
        }

        reprs
    }
}

impl Default for App {
    fn default() -> Self {
        App {
            creatures: Vec::new(),
            current_screen: CurrentScreen::Main,
            creature_list_state: ListState::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CurrentScreen {
    Main,
    Editing(CurrentlyEditing),
}

#[derive(Debug, Clone)]
pub enum CurrentlyEditing {
    Player,
    Monster,
}
