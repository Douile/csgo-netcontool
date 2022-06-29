use strum::EnumDiscriminants;

use super::{Damage, Status, UIState};

#[derive(Debug, EnumDiscriminants, PartialEq)]
pub enum Event {
    Command(String),
    ChangeUIState(UIState, UIState),
    Damage(Damage),
    MapChange(String),
    PlayerConnected(String),
    EnterBuyPeriod,
    Status(Status),
    ConVar(String, String),
    Tick(u8),
}

impl Event {
    pub fn is_variant(&self, discrimant: EventDiscriminants) -> bool {
        EventDiscriminants::from(self) == discrimant
    }
}
