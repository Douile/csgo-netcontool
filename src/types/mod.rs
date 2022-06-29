pub mod damage;
pub mod event;
pub mod game_mode;
pub mod state;
pub mod status;
pub mod ui_state;

pub use self::damage::*;
pub use self::event::*;
pub use self::game_mode::*;
pub use self::state::*;
pub use self::status::*;
pub use self::ui_state::UIState;

pub type GenericResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;
pub type StateListener = fn(State) -> ();
