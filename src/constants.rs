use std::time::Duration;

pub const BUF_SIZE: usize = 5012;
pub const NEWLINE: u8 = b'\n';
pub const PORT: u16 = 5555;
pub const TICK_TIME: Duration = Duration::from_millis(500);
pub const TICK_COMMAND: &[u8] = b"clan;incrementvar cl_hud_color 0 5 1\n";
