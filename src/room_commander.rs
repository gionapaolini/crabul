use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    oneshot,
};

use crate::{
    consts::{GameError, PlayerId},
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
            .send(RoomCommand::RemovePlayer { id, cmd_tx })
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
            .send(RoomCommand::SetPlayerReady { id, cmd_tx })
            .unwrap();
        cmd_rx.await.unwrap()
    }
}
