pub type GenericResult<T> = Result<T, Box<dyn std::error::Error>>;
#[derive(Debug)]
pub enum Event {
    Command(String),
    ChangeUIState(UIState, UIState),
    Damage(Damage),
    MapChange(String),
    PlayerConnected(String),
    EnterBuyPeriod,
    Status(Status),
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug)]
pub enum DamageDirection {
    Given,
    Taken,
}

#[derive(Debug)]
pub struct Damage {
    pub direction: DamageDirection,
    pub target: String,
    pub amount: u8,
    pub hits: u8,
}

impl TryFrom<&str> for Damage {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut value = value
            .strip_prefix("Damage ")
            .ok_or("Invalid damage string 1")?;

        let direction = if value.starts_with("Taken from") {
            value = &value[10..];
            DamageDirection::Taken
        } else if value.starts_with("Given to") {
            value = &value[8..];
            DamageDirection::Given
        } else {
            Err("Unknown damage direction")?
        };

        let value = value.strip_prefix(" \"").ok_or("Invalid damage string 3")?;

        let (target, value) = value.split_once("\"").ok_or("Invalid damage string 4")?;

        let value = value.strip_prefix(" - ").ok_or("Invalid damage string 5")?;

        let (amount, value) = value.split_once(" ").ok_or("Invalid damage string 6")?;

        let value = value.strip_prefix("in ").ok_or("Invalid damage string 7")?;

        let (hits, _) = value.split_once(" ").ok_or("Invalid damage string 8")?;

        Ok(Damage {
            direction,
            target: target.to_string(),
            amount: u8::from_str_radix(amount, 10).or(Err("Invalid damage amount"))?,
            hits: u8::from_str_radix(hits, 10).or(Err("Invalid damage hits amount"))?,
        })
    }
}

#[derive(Debug)]
pub enum Status {
    NotConnected,
    Connected(StatusData),
}

#[derive(Debug, derive_builder::Builder)]
pub struct StatusData {
    pub hostname: String,
    pub version: String,
    pub address: String,
    pub os: String,
    pub server_type: String,
    pub map: String,
    pub players: String,
}

impl StatusData {
    pub fn parse<T: std::io::BufRead>(hostname: String, reader: &mut T) -> GenericResult<Self> {
        let mut line: Vec<u8> = Vec::new();
        let mut builder = StatusDataBuilder::default();
        builder.hostname(hostname);

        loop {
            line.clear();
            let n = reader.read_until(b'\n', &mut line)?;
            if n == 0 {
                continue;
            }
            let line = String::from_utf8_lossy(&line[..n]);
            let line = line.trim();
            eprintln!("Parsing {:?}", line);

            if line == "#end" {
                break;
            }

            if let Some(version) = line.strip_prefix("version : ") {
                builder.version(version.trim().to_string());
                continue;
            }

            if let Some(address) = line.strip_prefix("udp/ip  : ") {
                builder.address(address.trim().to_string());
                continue;
            }

            if let Some(os) = line.strip_prefix("os      : ") {
                builder.os(os.trim().to_string());
                continue;
            }

            if let Some(server_type) = line.strip_prefix("type    : ") {
                builder.server_type(server_type.trim().to_string());
                continue;
            }

            if let Some(map) = line.strip_prefix("map     : ") {
                builder.map(map.trim().to_string());
                continue;
            }

            if let Some(players) = line.strip_prefix("players : ") {
                builder.players(players.trim().to_string());
                continue;
            }
        }

        builder.build().map_err(|e| e.into())
    }
}

#[derive(Debug)]
pub struct State {
    pub ui_state: UIState,
}

impl Default for State {
    fn default() -> Self {
        Self {
            ui_state: UIState::MainMenu,
        }
    }
}
