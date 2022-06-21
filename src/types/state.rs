use super::Status;
use super::UIState;

#[derive(Debug, Clone)]
pub struct State {
    pub ui_state: UIState,
    pub status: Status,
    pub map: Option<String>,
    pub round: u8,
    pub total_damage: u64,
    pub enabled: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            ui_state: UIState::MainMenu,
            status: Status::NotConnected,
            map: None,
            round: 0,
            total_damage: 0,
            enabled: false,
        }
    }
}

impl State {
    pub fn clear_game_data(&mut self) {
        self.map = None;
        self.round = 0;
        self.total_damage = 0;
    }
}
