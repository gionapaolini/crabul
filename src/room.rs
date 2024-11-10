use std::collections::HashMap;

use rand::{thread_rng, Rng};
use tokio::{
    spawn,
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
};

pub struct RoomCommander {
    tx_channel: UnboundedSender<RoomCommand>,
}

impl RoomCommander {
    pub async fn new_player(&self, name: String) -> (PlayerId, UnboundedReceiver<RoomEvent>) {
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
}

pub struct Player {
    id: PlayerId,
    name: PlayerName,
    tx: UnboundedSender<RoomEvent>
}

pub struct RoomServer {
    id: RoomId,
    tx_channel: UnboundedSender<RoomCommand>,
    rx_channel: UnboundedReceiver<RoomCommand>,
    players: HashMap<PlayerId, Player>,
}
impl RoomServer {
    pub fn new() -> RoomCommander {
        let (tx_channel, rx_channel) = mpsc::unbounded_channel();

        let room_server = RoomServer {
            id: thread_rng().gen::<RoomId>(),
            tx_channel: tx_channel.clone(),
            rx_channel,
            players: HashMap::with_capacity(6),
        };

        spawn(room_server.run());

        RoomCommander { tx_channel }
    }
    pub async fn run(mut self) {
        while let Some(cmd) = self.rx_channel.recv().await {
            match cmd {
                RoomCommand::AddPlayer { name, cmd_tx } => {
                    let (rx_channel, player_id) = self.new_player(name);
                    let _ = cmd_tx.send((player_id, rx_channel));
                }
                RoomCommand::RemovePlayer { id, cmd_tx } => {
                    self.remove_player(id);
                    let _ = cmd_tx.send(());
                },
            }
        }
    }

    fn new_player(&mut self, name: PlayerName) -> (UnboundedReceiver<RoomEvent>, u16) {
        let (tx_channel, rx_channel) = mpsc::unbounded_channel();
        let player_id = thread_rng().gen::<PlayerId>();

        self.players.insert(player_id, Player{
            id: player_id,
            name: name.clone(),
            tx: tx_channel
        });

        let event = RoomEvent::PlayerJoined {
            room_id: self.id,
            player_id,
            player_name: name,
        };

        self.send_all_players(event);

        (rx_channel, player_id)
    }

    fn remove_player(&mut self, id: PlayerId) {
        self.players.remove(&id);

        let event = RoomEvent::PlayerLeft(id);

        self.send_all_players(event);
    }

    fn send_all_players(&self, event: RoomEvent) {
        self.players.iter().for_each(|(_, player)| {
            let _ = player.tx.send(event.clone());
        });
    }
}

pub type RoomId = u16;
pub type PlayerId = u16;
pub type PlayerName = String;

#[derive(Clone)]
pub enum RoomEvent {
    PlayerJoined {
        room_id: RoomId,
        player_id: PlayerId,
        player_name: PlayerName,
    },
    PlayerLeft(PlayerId)
}

pub enum RoomCommand {
    AddPlayer {
        name: PlayerName,
        cmd_tx: oneshot::Sender<(PlayerId, UnboundedReceiver<RoomEvent>)>,
    },
    RemovePlayer {
        id: PlayerId,
        cmd_tx: oneshot::Sender<()>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_player() {
        let room_commander = RoomServer::new();
        let player_name = "name1";
        let (player_id, mut player_receiver) = room_commander.new_player(player_name.into()).await;

        let received_event = player_receiver.recv().await.unwrap();
        assert!(
            matches!(received_event, RoomEvent::PlayerJoined { room_id: _, player_id: received_player_id, player_name: received_player_name} if received_player_id==player_id && received_player_name==player_name)
        );
    }

    #[tokio::test]
    async fn new_player_previous_player_should_receive_the_join_event() {
        let room_commander = RoomServer::new();
        let (player_name_1, player_name_2) = ("name1", "name2");
        let (_, player_receiver_1) = room_commander.new_player(player_name_1.into()).await;
        let (player_id_2, _) = room_commander.new_player(player_name_2.into()).await;

        let received_event = get_nth_event(player_receiver_1, 2).await;
        assert!(
            matches!(received_event, RoomEvent::PlayerJoined { room_id: _, player_id: received_player_id, player_name: received_player_name} if received_player_id==player_id_2 && received_player_name==player_name_2)
        );
    }

    #[tokio::test]
    async fn remove_player() {
        let room_commander = RoomServer::new();
        let (player_name_1, player_name_2, player_name_3) = ("name1", "name2", "name3");
        let (_, player_receiver_1) = room_commander.new_player(player_name_1.into()).await;
        let (_, player_receiver_2) = room_commander.new_player(player_name_2.into()).await;
        let (player_id_3, _) = room_commander.new_player(player_name_3.into()).await;
        room_commander.remove_player(player_id_3).await;

        let received_event = get_nth_event(player_receiver_1, 4).await;
        assert!(
            matches!(received_event, RoomEvent::PlayerLeft(received_player_id) if received_player_id == player_id_3)
        );

        let received_event = get_nth_event(player_receiver_2, 3).await;      
        assert!(
            matches!(received_event, RoomEvent::PlayerLeft(received_player_id) if received_player_id == player_id_3)
        );
    }

    async fn get_nth_event(mut rcv: UnboundedReceiver<RoomEvent>, nth: u8) -> RoomEvent {
        for _ in 1..nth {
            rcv.recv().await.unwrap();
        }
        rcv.recv().await.unwrap()
    }
}
