use strum::EnumDiscriminants;
use tokio::io::{AsyncRead, AsyncReadExt};

use super::GenericResult;
use crate::LineReader;

#[derive(Debug, Clone, EnumDiscriminants)]
pub enum Status {
    NotConnected,
    Connected(StatusData),
}

impl PartialEq for Status {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Status {
    pub fn is_variant(&self, discrimant: StatusDiscriminants) -> bool {
        StatusDiscriminants::from(self) == discrimant
    }
}

#[derive(Debug, Clone, derive_builder::Builder)]
pub struct StatusData {
    pub hostname: String,
    pub host_type: HostType,
    pub version: String,
    #[builder(setter(into, strip_option), default)]
    pub address: Option<String>,
    pub os: String,
    pub server_type: String,
    pub map: String,
    pub players: Players,
    pub player_list: Vec<Player>,
}

#[derive(Debug, Clone)]
pub enum HostType {
    Official(String),
    Unofficial,
}

#[derive(Debug, Clone)]
pub struct Players {
    pub humans: u32,
    pub bots: u32,
    pub max: u32,
}

impl Players {
    pub fn total(&self) -> u32 {
        self.humans + self.bots
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub steam_id: String,
}

impl StatusData {
    pub async fn parse<T: AsyncRead + AsyncReadExt + Send>(
        hostname: String,
        reader: &mut LineReader<T>,
    ) -> GenericResult<Self> {
        let mut builder = StatusDataBuilder::default();
        builder.hostname(hostname.clone());

        builder.host_type(HostType::Unofficial);

        let mut player_list = Vec::new();
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
                if server_type.contains("official") {
                    if let Some(official) = hostname[..].strip_prefix("Valve CS:GO ") {
                        if let Some((official, _)) = official.split_once(" Server") {
                            builder.host_type(HostType::Official(official.to_string()));
                        }
                    }
                }
                continue;
            }

            if let Some(map) = line.strip_prefix("map     : ") {
                if let Some((map, _)) = map.split_once(' ') {
                    builder.map(map.to_string());
                } else {
                    builder.map(map.trim().to_string());
                }
                continue;
            }

            if let Some(line) = line.strip_prefix("players : ") {
                if let Some((humans, line)) = line.split_once("humans, ") {
                    if let Some((bots, line)) = line.split_once(" bots (") {
                        if let Some((max, _)) = line.split_once("/") {
                            builder.players(Players {
                                humans: u32::from_str_radix(humans.trim(), 10).unwrap(),
                                bots: u32::from_str_radix(bots.trim(), 10).unwrap(),
                                max: u32::from_str_radix(max.trim(), 10).unwrap(),
                            });
                            continue;
                        }
                    }
                }
            }

            if let Some(line) = line.strip_prefix("#") {
                if let Some((id, line)) = line.split_once('"') {
                    if let Some((name, line)) = line.split_once("\" ") {
                        if let Some((steam_id, _)) = line.split_once(' ') {
                            player_list.push(Player {
                                id: id.trim().to_string(),
                                name: name.trim().to_string(),
                                steam_id: steam_id.trim().to_string(),
                            });
                        }
                    }
                }
            }
        }

        builder.player_list(player_list);

        builder.build().map_err(|e| e.into())
    }
}
