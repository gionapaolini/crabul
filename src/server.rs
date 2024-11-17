use std::{collections::HashMap, io};

use serde::Serialize;
use tokio::{
    spawn,
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
};

use crate::{
    consts::RoomId,
    room::{commander::RoomCommander, server::RoomServer},
};

#[derive(Serialize)]
pub enum ServerError {
    RoomNotFound,
}

pub enum ServerCommand {
    NewRoom {
        cmd_tx: oneshot::Sender<RoomCommander>,
    },
    JoinRoom {
        room_id: RoomId,
        cmd_tx: oneshot::Sender<Result<RoomCommander, ServerError>>,
    },
    DestroyRoom {
        room_id: RoomId,
    },
}

#[derive(Clone)]
pub struct ServerCommander {
    tx_channel: UnboundedSender<ServerCommand>,
}

impl ServerCommander {
    pub async fn new_room(&self) -> RoomCommander {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        self.tx_channel
            .send(ServerCommand::NewRoom { cmd_tx })
            .unwrap();
        cmd_rx.await.unwrap()
    }
    pub async fn join_room(&self, room_id: RoomId) -> Result<RoomCommander, ServerError> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        self.tx_channel
            .send(ServerCommand::JoinRoom { room_id, cmd_tx })
            .unwrap();
        cmd_rx.await.unwrap()
    }
}

pub struct Server {
    rooms: HashMap<RoomId, RoomCommander>,
    tx_channel: UnboundedSender<ServerCommand>,
    rx_channel: UnboundedReceiver<ServerCommand>,
}

impl Server {
    pub fn new() -> (Self, ServerCommander) {
        let (tx_channel, rx_channel) = mpsc::unbounded_channel();
        (
            Self {
                rooms: HashMap::new(),
                tx_channel: tx_channel.clone(),
                rx_channel,
            },
            ServerCommander { tx_channel },
        )
    }
    pub async fn run(mut self) -> io::Result<()> {
        while let Some(msg) = self.rx_channel.recv().await {
            match msg {
                ServerCommand::NewRoom { cmd_tx } => {
                    let res = self.new_room();
                    let _ = cmd_tx.send(res);
                }
                ServerCommand::JoinRoom { room_id, cmd_tx } => {
                    let res = self.join_room(room_id);
                    let _ = cmd_tx.send(res);
                }
                ServerCommand::DestroyRoom { room_id } => {
                    self.destroy_room(room_id);
                }
            }
        }
        Ok(())
    }
    fn new_room(&mut self) -> RoomCommander {
        let (room_id, room_commander) = RoomServer::start();
        self.rooms.insert(room_id, room_commander.clone());
        spawn(Self::remove_room(
            self.tx_channel.clone(),
            room_id,
            room_commander.clone(),
        ));

        room_commander
    }

    fn destroy_room(&mut self, room_id: RoomId) {
        self.rooms.remove(&room_id);
    }

    fn join_room(&mut self, room_id: RoomId) -> Result<RoomCommander, ServerError> {
        let room = self
            .rooms
            .get_mut(&room_id)
            .ok_or(ServerError::RoomNotFound)?;
        Ok(room.clone())
    }

    async fn remove_room(
        tx_channel: UnboundedSender<ServerCommand>,
        room_id: RoomId,
        room_channel: RoomCommander,
    ) {
        room_channel.tx_channel.closed().await;
        let _ = tx_channel.send(ServerCommand::DestroyRoom { room_id });
    }
}

#[cfg(test)]
mod tests {
    use tokio::spawn;

    use crate::room::events::RoomEvent;

    use super::*;

    #[tokio::test]
    async fn create_room() {
        let (mut server, _) = Server::new();
        assert!(server.rooms.is_empty());
        let _ = server.new_room();
        assert!(server.rooms.len() == 1);
    }

    #[tokio::test]
    async fn join_and_auto_delete_room() {
        let (server, server_commander) = Server::new();
        spawn(server.run());
        let room_commander = server_commander.new_room().await;
        let (_, mut player) = room_commander.new_player("test1".into()).await.unwrap();
        if let Ok(RoomEvent::PlayerJoined {
            room_id,
            player_id,
            player_name: _,
            player_list: _,
        }) = player.try_recv()
        {
            room_commander.remove_player(player_id).await;
            let res = server_commander.join_room(room_id).await;
            assert!(matches!(res, Err(ServerError::RoomNotFound)))
        } else {
            panic!("Did not receive player joined event");
        }
    }
}
