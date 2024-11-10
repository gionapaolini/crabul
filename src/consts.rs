use std::time::Duration;

#[derive(Debug)]
pub enum GameError {
    NameAlreadyExists,
    NotEnoughPlayers,
    TooManyPlayers,
    CannotAddNewPlayers,
    CannotStartTheGameFromCurrentState,
}

pub const MIN_PLAYERS: usize = 2;
pub const MAX_PLAYERS: usize = 6;
pub const PEEKING_PHASE_COUNTDOWN: Duration = Duration::from_secs(30);

pub type RoomId = u16;
pub type PlayerId = u16;
pub type PlayerName = String;
