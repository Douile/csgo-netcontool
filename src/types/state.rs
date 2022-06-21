use super::Status;
use super::UIState;

#[derive(Debug)]
pub struct State {
    pub ui_state: UIState,
    pub status: Status,
    pub enabled: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            ui_state: UIState::MainMenu,
            status: Status::NotConnected,
            enabled: false,
        }
    }
}
