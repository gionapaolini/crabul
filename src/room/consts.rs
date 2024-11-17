use std::time::Duration;

pub const MIN_PLAYERS: usize = 2;
pub const MAX_PLAYERS: usize = 6;
pub const PEEKING_PHASE_COUNTDOWN: Duration = Duration::from_secs(10);
pub const TURN_COUNTDOWN: Duration = Duration::from_secs(600);
pub const FINALIZE_GAME_COUNTDOWN: Duration = Duration::from_secs(5);
