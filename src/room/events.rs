use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    consts::{PlayerId, PlayerName, RoomId},
    deck::Card,
};

use super::server::{DuplicateCardResult, FinalScore, Power};

#[derive(Deserialize, Serialize, Clone)]
pub enum RoomEvent {
    PlayerJoined {
        room_id: RoomId,
        player_id: PlayerId,
        player_name: PlayerName,
        player_list: HashMap<PlayerId, PlayerName>,
    },
    PlayerLeft(PlayerId),
    GameStarted,
    PlayerTurn(PlayerId),
    PeekingPhaseStarted((Card, Card)),
    PlayerIsReady(PlayerId),
    CardWasDrawn(PlayerId),
    DrawnCard(Card),
    CardSwapped(PlayerId, usize),
    CardDiscarded(PlayerId, Card),
    PowerActivated(PlayerId, Power),
    PeekedCard(Card),
    PowerUsed(
        Power,
        PlayerId,
        Option<usize>,
        Option<PlayerId>,
        Option<usize>,
    ),
    DuplicateCardAttempt(PlayerId, PlayerId, usize, Option<Card>, DuplicateCardResult),
    CardReplaced(PlayerId, usize, PlayerId, usize),
    PlayerWentCrabul(PlayerId),
    GameTerminated(FinalScore),
    TurnEndedByTimeout(PlayerId),
    PowerDiscarded(PlayerId, Power),
    ForcedBlindSwap(PlayerId, usize, PlayerId, usize),
}
