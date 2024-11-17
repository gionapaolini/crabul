use tokio::sync::{mpsc::UnboundedReceiver, oneshot};

use crate::consts::{GameError, PlayerId, PlayerName};

use super::events::RoomEvent;

pub enum RoomCommand {
    AddPlayer {
        name: PlayerName,
        cmd_tx: oneshot::Sender<Result<(PlayerId, UnboundedReceiver<RoomEvent>), GameError>>,
    },
    RemovePlayer {
        player_id: PlayerId,
        cmd_tx: oneshot::Sender<()>,
    },
    StartGame {
        cmd_tx: oneshot::Sender<Result<(), GameError>>,
    },
    SetPlayerReady {
        player_id: PlayerId,
        cmd_tx: oneshot::Sender<Result<(), GameError>>,
    },
    NextTurn,
    GoCrabul {
        player_id: PlayerId,
        cmd_tx: oneshot::Sender<Result<(), GameError>>,
    },
    DrawCard {
        player_id: PlayerId,
        cmd_tx: oneshot::Sender<Result<(), GameError>>,
    },
    SwapCard {
        player_id: PlayerId,
        card_idx: usize,
        cmd_tx: oneshot::Sender<Result<(), GameError>>,
    },
    DiscardCard {
        player_id: PlayerId,
        cmd_tx: oneshot::Sender<Result<(), GameError>>,
    },
    PeekOwnCard {
        player_id: PlayerId,
        card_idx: usize,
        cmd_tx: oneshot::Sender<Result<(), GameError>>,
    },
    PeekOtherCard {
        player_id: PlayerId,
        other_player_id: PlayerId,
        other_card_idx: usize,
        cmd_tx: oneshot::Sender<Result<(), GameError>>,
    },
    BlindSwap {
        player_id: PlayerId,
        card_idx: usize,
        other_player_id: PlayerId,
        other_card_idx: usize,
        cmd_tx: oneshot::Sender<Result<(), GameError>>,
    },
    CheckAndSwapStage1 {
        player_id: PlayerId,
        other_player_id: PlayerId,
        other_card_idx: usize,
        cmd_tx: oneshot::Sender<Result<(), GameError>>,
    },
    CheckAndSwapStage2 {
        player_id: PlayerId,
        card_idx: Option<usize>,
        cmd_tx: oneshot::Sender<Result<(), GameError>>,
    },
    ThrowSameCard {
        player_id: PlayerId,
        picked_player_id: PlayerId,
        picked_card_idx: usize,
        cmd_tx: oneshot::Sender<Result<(), GameError>>,
    },
    SelectCardToGiveAway {
        player_id: PlayerId,
        card_idx: usize,
        cmd_tx: oneshot::Sender<Result<(), GameError>>,
    },
    StopRoomServer,
    ForceEndTurn(PlayerId),
    FinalizeGame,
}