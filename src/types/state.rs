use super::game_mode::{GameMode, GameType};
use super::Status;
use super::UIState;

#[derive(Debug, Clone)]
pub struct State {
    pub ui_state: UIState,
    pub status: Status,
    pub map: Option<String>,
    pub round: u8,
    pub total_damage_given: u64,
    pub total_damage_taken: u64,
    pub game_type: GameType,
    pub game_mode: GameMode,
    pub enabled: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            ui_state: UIState::MainMenu,
            status: Status::NotConnected,
            map: None,
            round: 0,
            total_damage_given: 0,
            total_damage_taken: 0,
            game_type: GameType::Classic,
            game_mode: GameMode::Casual,
            enabled: false,
        }
    }
}

impl State {
    pub fn clear_game_data(&mut self, map: Option<String>) {
        self.map = map;
        self.round = 0;
        self.total_damage_given = 0;
        self.total_damage_taken = 0;
    }
}
