use tokio::io::AsyncRead;

use crate::reader::LineReader;
use crate::types::{Damage, Event, GenericResult, Status, StatusData};

pub async fn stream_reader<T: AsyncRead + Send>(
    mut line_reader: LineReader<T>,
    chan: async_channel::Sender<Event>,
) -> GenericResult<()> {
    loop {
        let line = line_reader.read_line().await?;
        let line = line.trim();

        if line.len() == 0 {
            continue;
        }

        println!(": {:?}", line);

        if let Some(data) = line.strip_prefix("ChangeGameUIState:") {
            let mut parts = data.split("->");
            let state = (
                parts.next().unwrap().trim().try_into(),
                parts.next().unwrap().trim().try_into(),
            );
            match state {
                (Ok(from_state), Ok(to_state)) => {
                    chan.send(Event::ChangeUIState(from_state, to_state))
                        .await?;
                }
                _ => {
                    eprintln!("Error {:?}", state);
                }
            }
            continue;
        }

        if let Some(map) = line.strip_prefix("Map: ") {
            chan.send(Event::MapChange(map.to_string())).await?;
            continue;
        }

        if let Some(player) = line.strip_suffix(" connected.") {
            chan.send(Event::PlayerConnected(player.to_string()))
                .await?;
            continue;
        }

        if line == "EVERYONE CAN BUY!" {
            chan.send(Event::EnterBuyPeriod).await?;
            continue;
        }

        if line == "Not connected to server" {
            chan.send(Event::Status(Status::NotConnected)).await?;
            continue;
        }

        if let Some(hostname) = line.strip_prefix("hostname: ") {
            if let Ok(status) = StatusData::parse(hostname.to_string(), &mut line_reader).await {
                chan.send(Event::Status(Status::Connected(status))).await?;
            }
            continue;
        }

        if let Ok(damage) = Damage::try_from(line) {
            chan.send(Event::Damage(damage)).await?;
        } else if line.starts_with("Damage") {
            // FIXME: Ignore
            eprintln!("{:?} {:?}", line, Damage::try_from(line));
        }

        if let Some(command) = line.strip_prefix("??? ") {
            chan.send(Event::Command(command.to_string())).await?;
        }
    }
}
