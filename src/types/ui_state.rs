#[derive(Debug, PartialEq, Clone)]
pub enum UIState {
    MainMenu,
    LoadingScreen,
    InGame,
    PauseMenu,
}

impl TryFrom<&str> for UIState {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "CSGO_GAME_UI_STATE_MAINMENU" => Ok(UIState::MainMenu),
            "CSGO_GAME_UI_STATE_LOADINGSCREEN" => Ok(UIState::LoadingScreen),
            "CSGO_GAME_UI_STATE_INGAME" => Ok(UIState::InGame),
            "CSGO_GAME_UI_STATE_PAUSEMENU" => Ok(UIState::PauseMenu),
            _ => Err(format!("Unknown game state \"{}\"", value).to_string()),
        }
    }
}
