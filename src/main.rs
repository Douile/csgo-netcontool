use std::net::SocketAddr;
use std::time::Duration;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::sleep;

pub mod constants;
pub mod reader;
pub mod stream_reader;
pub mod types;
use crate::constants::{PORT, TICK_COMMAND, TICK_TIME};
use crate::reader::LineReader;
use crate::stream_reader::stream_reader;
use crate::types::{Event, EventDiscriminants, GenericResult, State, StatusDiscriminants, UIState};

#[tokio::main]
async fn main() -> GenericResult<()> {
    let mut state = State::default();
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

    eprintln!("Connected...");

    {
        let tx = tx.clone();
        tokio::spawn(async move {
            stream_reader(line_reader, tx).await.unwrap();
            ()
        });
    }

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
                    _ => {
                        eprintln!("Sending command {:?}", command);
                        let mut cmd_conn = TcpStream::connect(&addr).await?;
                        cmd_conn
                            .write_all(&format!("{}\n", command).into_bytes())
                            .await?;
                    }
                },
                Event::ChangeUIState(_, new_state) => {
                    state.ui_state = new_state;
                }
                Event::Status(new_status) => {
                    state.status = new_status;
                }
                Event::Tick(_)
                    if state.enabled
                        && state.ui_state == UIState::InGame
                        && state.status.is_variant(StatusDiscriminants::Connected) =>
                {
                    eprintln!("InGame tick");
                    let mut cmd_conn = TcpStream::connect(&addr).await?;
                    cmd_conn.write_all(TICK_COMMAND).await?;
                }
                _ => {}
            }
        }
    }
}
