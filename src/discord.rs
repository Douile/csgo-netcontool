use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use discord_presence::models::{Activity, ActivityAssets, ActivityButton, ActivityParty};
use discord_presence::Client;

use crate::types::{HostType, State, StateListener, Status, UIState};

static mut SENDER: Option<Mutex<Sender<State>>> = None;

fn client_thread(rx: Receiver<State>) {
    eprintln!("RPC starting");
    let mut client = Client::new(425776052565049354);
    client.start();

    eprintln!("RPC ready");
    let mut state = rx.recv().unwrap();
    let mut updated = true;
    loop {
        if updated {
            eprintln!("RPC received {:?}", state);
            let state_string = match state.ui_state {
                UIState::MainMenu => Some(String::from("In the main menu")),
                UIState::LoadingScreen => Some(String::from("Loading...")),
                UIState::InGame => {
                    Some(format!("In {} game", state.game_mode.to_string()).to_string())
                }
                UIState::PauseMenu => Some(String::from("Tabbed out of a game")),
            };

            let damage_string = if state.round > 0 {
                format!("{} ADR", state.total_damage_given / state.round as u64)
            } else {
                format!("{} DMG", state.total_damage_given)
            };

            let details_string = match &state.map {
                Some(map) => Some(format!("Playing {} ({})", map, damage_string).to_string()),
                None => Some(String::from("Idling...")),
            };

            let mut buttons = Vec::new();

            if let Status::Connected(data) = &state.status {
                if data.address.is_some() {
                    buttons.push(ActivityButton {
                        label: Some(String::from("Join")),
                        url: Some(
                            format!("steam://connect/{}", data.address.as_ref().unwrap())
                                .to_string(),
                        ),
                    });
                }

                if let HostType::Official(region) = &data.host_type {
                    buttons.push(ActivityButton {
                        label: Some(format!("Official ({})", region).to_string()),
                        url: Some(String::from("https://bogus")),
                    });
                }
            }

            let party = match &state.status {
                Status::Connected(data) => Some(ActivityParty {
                    size: Some((data.players.total(), data.players.max)),
                    ..ActivityParty::default()
                }),
                _ => Some(ActivityParty::default()),
            };

            let image_detail_string = format!(
                "{}/{} DMG, round {}",
                state.total_damage_given, state.total_damage_taken, state.round
            )
            .to_string();

            if let Err(e) = client.set_activity(|_| {
                Activity {
            state: state_string,
            details: details_string,
            assets: Some(ActivityAssets {
                large_image: Some(String::from(
                    "https://i.pinimg.com/originals/3f/73/47/3f7347c1a4a72c1b39bc14138c377737.png",
                )),
                large_text: Some(image_detail_string),
                ..ActivityAssets::default()
            }),
            buttons: if buttons.len() > 0 { Some(buttons) } else { None},
            party,
            ..Activity::default()
        }
            }) {
                eprintln!("RPC error: {:?}", e);
            }
        }
        // Sleep to avoid spamming discord
        thread::sleep(Duration::from_secs(1));

        updated = false;
        while let Ok(new_state) = rx.try_recv() {
            state = new_state;
            updated = true;
        }
    }
}

fn listener(state: State) {
    unsafe {
        if let Some(mutex) = &SENDER {
            let lock = mutex.lock().unwrap();
            lock.send(state).unwrap();
        }
    }
}

pub fn register_listener(listeners: &mut Vec<StateListener>) {
    let (tx, rx) = channel();
    unsafe {
        SENDER = Some(Mutex::new(tx));
    }
    listeners.push(listener);
    thread::spawn(move || {
        client_thread(rx);
    });
}
