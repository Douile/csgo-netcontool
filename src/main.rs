use std::net::SocketAddr;
use std::time::Duration;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::sleep;

pub mod constants;
#[cfg(feature = "rpc")]
mod discord;
pub mod reader;
pub mod stream_reader;
pub mod types;
use crate::constants::{PORT, TICK_COMMAND, TICK_TIME};
use crate::reader::LineReader;
use crate::stream_reader::stream_reader;
use crate::types::{
    DamageDirection, Event, EventDiscriminants, GameMode, GameType, GenericResult, State,
    StateListener, Status, StatusDiscriminants, UIState,
};

#[tokio::main]
async fn main() -> GenericResult<()> {
    let (tx, rx) = async_channel::unbounded();

    // Make connection
    eprintln!("Making TCP Connection");
    let stream: TcpStream;
    let addr = SocketAddr::from(([127, 0, 0, 1], PORT));
    loop {
        match TcpStream::connect(&addr).await {
            Ok(s) => {
                stream = s;
                break;
            }
            _ => {}
        }
        sleep(Duration::from_secs(1)).await;
    }
    // Drop writer here as it is necessary to create a new connection each time we want to write.
    // This is due to CS:GO crashing if 2 or more writes are made to a socket in between each read,
    // but reads cannot be made without console output
    let (rd, _) = tokio::io::split(stream);
    let line_reader = LineReader::new(rd);

    let mut listeners: Vec<StateListener> = Vec::new();
    #[cfg(feature = "rpc")]
    discord::register_listener(&mut listeners);

    eprintln!("Connected...");

    {
        let tx = tx.clone();
        tokio::spawn(async move {
            stream_reader(line_reader, tx).await.unwrap();
            ()
        });
    }

    let mut state = State::default();
    call_state_update_listeners(&listeners, &state);

    tokio::spawn(async move {
        let mut tick_no = 0u8;
        loop {
            tx.send(Event::Tick(tick_no)).await.unwrap();
            tick_no = u8::wrapping_add(tick_no, 1);
            sleep(TICK_TIME).await;
        }
    });

    loop {
        let event = rx.recv().await;
        if let Ok(event) = event {
            if !event.is_variant(EventDiscriminants::Tick) {
                eprintln!("{:?}", event);
            }

            match event {
                Event::Command(command) => match command.as_str() {
                    "toggle" => state.enabled = !state.enabled,
                    "start" => {
                        state.clear_game_data(state.map.clone());
                        call_state_update_listeners(&listeners, &state);
                    }
                    "addround" => {
                        state.round += 1;
                        call_state_update_listeners(&listeners, &state);
                    }
                    _ => {
                        eprintln!("Sending command {:?}", command);
                        send_command(&addr, &format!("{}\n", command).into_bytes()).await?;
                    }
                },
                Event::ChangeUIState(_, new_state) => {
                    state.ui_state = new_state;
                    if state.ui_state == UIState::MainMenu {
                        state.clear_game_data(None);
                    }
                    if state.ui_state == UIState::InGame
                        && state.status.is_variant(StatusDiscriminants::NotConnected)
                    {
                        send_command(&addr, b"status\n").await?;
                    }
                    call_state_update_listeners(&listeners, &state);
                }
                Event::Status(new_status) => {
                    state.status = new_status;
                    if let Status::Connected(data) = &state.status {
                        state.map = Some(data.map.clone());
                    } else {
                        state.clear_game_data(None);
                    }
                    call_state_update_listeners(&listeners, &state);
                }
                Event::MapChange(map) => {
                    state.clear_game_data(Some(map));
                    call_state_update_listeners(&listeners, &state);
                }
                Event::EnterBuyPeriod => {
                    state.round += 1;
                    call_state_update_listeners(&listeners, &state);
                }
                Event::Damage(damage) => {
                    if damage.direction == DamageDirection::Given {
                        state.total_damage_given += u8::max(damage.amount, 100) as u64;
                    } else {
                        state.total_damage_taken += u8::max(damage.amount, 100) as u64;
                    }
                    call_state_update_listeners(&listeners, &state);
                }
                Event::ConVar(name, value) => {
                    if name == "game_type" {
                        if let Ok(value) = u8::from_str_radix(&value, 10) {
                            if let Some(game_type) = GameType::try_from(value) {
                                state.game_type = game_type;
                                call_state_update_listeners(&listeners, &state);
                                continue;
                            }
                        }
                    }
                    if name == "game_mode" {
                        if let Ok(value) = u8::from_str_radix(&value, 10) {
                            if let Some(game_mode) =
                                GameMode::try_from((state.game_type.clone(), value))
                            {
                                state.game_mode = game_mode;
                                call_state_update_listeners(&listeners, &state);
                                continue;
                            }
                        }
                    }
                }
                Event::Tick(_)
                    if state.enabled
                        && state.ui_state == UIState::InGame
                        && state.status.is_variant(StatusDiscriminants::Connected) =>
                {
                    eprintln!("InGame tick");
                    send_command(&addr, TICK_COMMAND).await?;
                }
                _ => {}
            }
        }
    }
}

async fn send_command(addr: &SocketAddr, data: &[u8]) -> tokio::io::Result<()> {
    let mut cmd_conn = TcpStream::connect(addr).await?;
    cmd_conn.write_all(data).await?;
    Ok(())
}

fn call_state_update_listeners(listeners: &Vec<StateListener>, state: &State) -> () {
    for listener in listeners {
        let state = state.clone();
        listener(state);
    }
}
