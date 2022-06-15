use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::net::{SocketAddr, TcpStream};
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::time::Duration;

pub mod types;
use self::types::{Damage, Event, GenericResult, State, Status, StatusData, UIState};

const PORT: u16 = 5555;

fn main() -> GenericResult<()> {
    // Make connection
    eprintln!("Making TCP Connection");
    let stream: TcpStream;
    loop {
        match TcpStream::connect(SocketAddr::from(([127, 0, 0, 1], PORT))) {
            Ok(s) => {
                stream = s;
                break;
            }
            _ => {}
        }
        thread::sleep(Duration::from_secs(1));
    }

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = BufWriter::new(stream);

    eprintln!("Connected...");

    let state = State::default();
    let state_lock = Arc::new(RwLock::new(state));

    let (tx, rx) = mpsc::channel();

    let tx_clone = tx.clone();
    thread::spawn(move || {
        stream_reader(&mut reader, tx_clone).unwrap();
    });

    {
        let state_lock = state_lock.clone();
        thread::spawn(move || {
            tick(state_lock, tx).unwrap();
        });
    }

    loop {
        let event = rx.recv()?;
        eprintln!("Event {:?}", event);

        match event {
            Event::ChangeUIState(_, to_state) => {
                let mut state = state_lock.write().unwrap();
                state.ui_state = to_state;
            }
            Event::Command(command) => {
                write!(writer, "{}\n", command)?;
            }
            _ => {}
        }
    }
}

fn stream_reader<T: BufRead>(reader: &mut T, chan: mpsc::Sender<Event>) -> GenericResult<()> {
    let mut line: Vec<u8> = Vec::new();
    loop {
        let n = reader.read_until(b'\n', &mut line)?;
        if n == 0 {
            continue;
        }

        let line = String::from_utf8_lossy(&line);
        let line = line.trim();

        if line.len() == 0 {
            continue;
        }

        println!("{:?}", line);

        if let Some(data) = line.strip_prefix("ChangeGameUIState:") {
            let mut parts = data.split("->");
            let state = (
                parts.next().unwrap().trim().try_into(),
                parts.next().unwrap().trim().try_into(),
            );
            match state {
                (Ok(from_state), Ok(to_state)) => {
                    chan.send(Event::ChangeUIState(from_state, to_state))?;
                }
                _ => {
                    eprintln!("Error {:?}", state);
                }
            }
            continue;
        }

        if let Some(map) = line.strip_prefix("Map: ") {
            chan.send(Event::MapChange(map.to_string()))?;
            continue;
        }

        if let Some(player) = line.strip_suffix(" connected.") {
            chan.send(Event::PlayerConnected(player.to_string()))?;
            continue;
        }

        if line == "EVERYONE CAN BUY!" {
            chan.send(Event::EnterBuyPeriod)?;
            continue;
        }

        if line == "Not connected to server" {
            chan.send(Event::Status(Status::NotConnected))?;
            continue;
        }

        if let Some(hostname) = line.strip_prefix("hostname: ") {
            if let Ok(status) = StatusData::parse(hostname.to_string(), reader) {
                chan.send(Event::Status(Status::Connected(status)))?;
            }
            continue;
        }

        if let Ok(damage) = Damage::try_from(line) {
            chan.send(Event::Damage(damage))?;
        } else if line.starts_with("Damage") {
            // FIXME: Ignore
            eprintln!("{:?} {:?}", line, Damage::try_from(line));
        }
    }
}

fn tick(state_lock: Arc<RwLock<State>>, chan: mpsc::Sender<Event>) -> GenericResult<()> {
    eprintln!("Tick");
    let mut tick: u16 = 0;
    return Ok(());
    loop {
        {
            let state = state_lock.read().unwrap();
            eprintln!("Tick {:?}", state);
            if state.ui_state == UIState::InGame {
                chan.send(Event::Command(String::from("say test")))?;
            }
        }
        tick = u16::wrapping_add(tick, 1);
        thread::sleep(Duration::from_secs(1));
    }
}
