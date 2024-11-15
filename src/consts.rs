use std::time::Duration;

use serde::Serialize;

#[derive(Serialize, Debug)]
pub enum GameError {
    NameAlreadyExists,
    EmptyName,
    NotEnoughPlayers,
    TooManyPlayers,
    OperationNotAllowedAtCurrentState,
    InvalidCardIndex,
    UnableToParseCommand,
}

pub const MIN_PLAYERS: usize = 2;
pub const MAX_PLAYERS: usize = 6;
pub const PEEKING_PHASE_COUNTDOWN: Duration = Duration::from_secs(1);
pub const TURN_COUNTDOWN: Duration = Duration::from_secs(600);

pub type RoomId = u16;
pub type PlayerId = u16;
pub type PlayerName = String;
