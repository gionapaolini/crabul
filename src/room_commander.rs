use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    oneshot,
};

use crate::{
    consts::{GameError, PlayerId},
    deck::Card,
    room::{RoomCommand, RoomEvent},
};

pub struct RoomCommander {
    tx_channel: UnboundedSender<RoomCommand>,
}

impl RoomCommander {
    pub fn new(tx_channel: UnboundedSender<RoomCommand>) -> Self {
        Self { tx_channel }
    }
    pub async fn new_player(
        &self,
        name: String,
    ) -> Result<(PlayerId, UnboundedReceiver<RoomEvent>), GameError> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        self.tx_channel
            .send(RoomCommand::AddPlayer { name, cmd_tx })
            .unwrap();
        cmd_rx.await.unwrap()
    }
    pub async fn remove_player(&self, id: PlayerId) {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        self.tx_channel
            .send(RoomCommand::RemovePlayer {
                player_id: id,
                cmd_tx,
            })
            .unwrap();
        cmd_rx.await.unwrap();
    }
    pub async fn start_game(&self) -> Result<(), GameError> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        self.tx_channel
            .send(RoomCommand::StartGame { cmd_tx })
            .unwrap();
        cmd_rx.await.unwrap()
    }
    pub async fn set_player_ready(&self, id: PlayerId) -> Result<(), GameError> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        self.tx_channel
            .send(RoomCommand::SetPlayerReady {
                player_id: id,
                cmd_tx,
            })
            .unwrap();
        cmd_rx.await.unwrap()
    }
    pub async fn draw_card(&self, id: PlayerId) -> Result<Card, GameError> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        self.tx_channel
            .send(RoomCommand::DrawCard {
                player_id: id,
                cmd_tx,
            })
            .unwrap();
        cmd_rx.await.unwrap()
    }
    pub async fn swap_card(&self, id: PlayerId, card_idx: usize) -> Result<(), GameError> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        self.tx_channel
            .send(RoomCommand::SwapCard {
                player_id: id,
                card_idx,
                cmd_tx,
            })
            .unwrap();
        cmd_rx.await.unwrap()
    }
    pub async fn discard_card(&self, id: PlayerId) -> Result<(), GameError> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        self.tx_channel
            .send(RoomCommand::DiscardCard {
                player_id: id,
                cmd_tx,
            })
            .unwrap();
        cmd_rx.await.unwrap()
    }
    pub async fn peek_own_card(&self, id: PlayerId, card_idx: usize) -> Result<(), GameError> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        self.tx_channel
            .send(RoomCommand::PeekOwnCard {
                player_id: id,
                card_idx,
                cmd_tx,
            })
            .unwrap();
        cmd_rx.await.unwrap()
    }

    pub async fn peek_other_card(
        &self,
        player_id: PlayerId,
        other_player_id: PlayerId,
        other_card_idx: usize,
    ) -> Result<(), GameError> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        self.tx_channel
            .send(RoomCommand::PeekOtherCard {
                player_id,
                other_player_id,
                other_card_idx,
                cmd_tx,
            })
            .unwrap();
        cmd_rx.await.unwrap()
    }

    pub async fn blind_swap(
        &self,
        player_id: PlayerId,
        card_idx: usize,
        other_player_id: PlayerId,
        other_card_idx: usize,
    ) -> Result<(), GameError> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        self.tx_channel
            .send(RoomCommand::BlindSwap {
                player_id,
                card_idx,
                other_player_id,
                other_card_idx,
                cmd_tx,
            })
            .unwrap();
        cmd_rx.await.unwrap()
    }
}
