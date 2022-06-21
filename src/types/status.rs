use strum::EnumDiscriminants;
use tokio::io::{AsyncRead, AsyncReadExt};

use super::GenericResult;
use crate::LineReader;

#[derive(Debug, PartialEq, EnumDiscriminants)]
pub enum Status {
    NotConnected,
    Connected(StatusData),
}

impl Status {
    pub fn is_variant(&self, discrimant: StatusDiscriminants) -> bool {
        StatusDiscriminants::from(self) == discrimant
    }
}

#[derive(Debug, PartialEq, derive_builder::Builder)]
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
    pub async fn parse<T: AsyncRead + AsyncReadExt + Send>(
        hostname: String,
        reader: &mut LineReader<T>,
    ) -> GenericResult<Self> {
        let mut builder = StatusDataBuilder::default();
        builder.hostname(hostname);

        loop {
            let line = reader.read_line().await?;
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
