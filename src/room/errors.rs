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
