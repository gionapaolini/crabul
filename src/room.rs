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
}

pub struct RoomServer {
    id: RoomId,
    tx_channel: UnboundedSender<RoomCommand>,
    rx_channel: UnboundedReceiver<RoomCommand>,
    players: Vec<UnboundedSender<RoomEvent>>,
}
impl RoomServer {
    pub fn new() -> RoomCommander {
        let (tx_channel, rx_channel) = mpsc::unbounded_channel();

        let room_server = RoomServer {
            id: thread_rng().gen::<RoomId>(),
            tx_channel: tx_channel.clone(),
            rx_channel,
            players: vec![],
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
            }
        }
    }

    fn new_player(&mut self, name: PlayerName) -> (UnboundedReceiver<RoomEvent>, u16) {
        let (tx_channel, rx_channel) = mpsc::unbounded_channel();
        let player_id = thread_rng().gen::<PlayerId>();

        self.players.push(tx_channel);

        let event = RoomEvent::PlayerJoined {
            room_id: self.id,
            player_id,
            player_name: name,
        };

        self.send_all_players(event);

        (rx_channel, player_id)
    }

    fn send_all_players(&self, event: RoomEvent) {
        self.players.iter().for_each(|tx| {
            let _ = tx.send(event.clone());
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
}

pub enum RoomCommand {
    AddPlayer {
        name: PlayerName,
        cmd_tx: oneshot::Sender<(PlayerId, UnboundedReceiver<RoomEvent>)>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_room() {
        let room_commander = RoomServer::new();
        let player_name = "name1";
        let (player_id, mut player_receiver) = room_commander.new_player(player_name.into()).await;

        let received_event = player_receiver.recv().await.unwrap();
        assert!(
            matches!(received_event, RoomEvent::PlayerJoined { room_id: _, player_id: received_player_id, player_name: received_player_name} if player_id==received_player_id && received_player_name==player_name)
        );
    }

    #[tokio::test]
    async fn join_room() {
        let room_commander = RoomServer::new();
        let (player_name_1, player_name_2) = ("name1", "name2");
        let (_, mut player_receiver_1) = room_commander.new_player(player_name_1.into()).await;
        let (player_id_2, _) = room_commander.new_player(player_name_2.into()).await;

        let _ = player_receiver_1.recv().await.unwrap(); //discard first
        let received_event = player_receiver_1.recv().await.unwrap();
        assert!(
            matches!(received_event, RoomEvent::PlayerJoined { room_id: _, player_id: received_player_id, player_name: received_player_name} if player_id_2==received_player_id && received_player_name==player_name_2)
        );
    }
}
