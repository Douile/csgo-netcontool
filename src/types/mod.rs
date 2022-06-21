pub mod damage;
pub mod event;
pub mod state;
pub mod status;
pub mod ui_state;

pub use self::damage::{Damage, DamageDirection};
pub use self::event::{Event, EventDiscriminants};
pub use self::state::State;
pub use self::status::{Status, StatusData, StatusDiscriminants};
pub use self::ui_state::UIState;

pub type GenericResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;
