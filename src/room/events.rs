use std::collections::HashMap;

use serde::Serialize;

use crate::{consts::{PlayerId, PlayerName, RoomId}, deck::Card, room_old::{FinalScore, Power, SameCardResult}};


#[derive(Serialize, Clone)]
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
    SameCardAttempt(PlayerId, PlayerId, usize, Option<Card>, SameCardResult),
    CardReplaced(PlayerId, usize, PlayerId, usize),
    PlayerWentCrabul(PlayerId),
    GameTerminated(FinalScore),
    TurnEndedByTimeout(PlayerId),
    PowerDiscarded(PlayerId, Power),
    ForcedBlindSwap(PlayerId, usize, PlayerId, usize),
}