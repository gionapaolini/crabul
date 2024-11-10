use std::collections::HashMap;

use rand::{thread_rng, Rng};
use tokio::{
    spawn,
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
};

pub const MIN_PLAYERS: usize = 2;
pub const MAX_PLAYERS: usize = 6;

#[derive(Debug)]
pub enum GameError {
    NameAlreadyExists,
    NotEnoughPlayers,
    TooManyPlayers
}

pub struct RoomCommander {
    tx_channel: UnboundedSender<RoomCommand>,
}

impl RoomCommander {
    pub async fn new_player(&self, name: String) -> Result<(PlayerId, UnboundedReceiver<RoomEvent>), GameError> {
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
}

pub struct Player {
    id: PlayerId,
    name: PlayerName,
    tx: UnboundedSender<RoomEvent>,
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
                    let res = self.new_player(name);
                    let _ = cmd_tx.send(res);
                }
                RoomCommand::RemovePlayer { id, cmd_tx } => {
                    self.remove_player(id);
                    let _ = cmd_tx.send(());
                }
                RoomCommand::StartGame { cmd_tx } => {
                    if self.players.len() < MIN_PLAYERS {
                        let _ = cmd_tx.send(Err(GameError::NotEnoughPlayers));
                    }
                }
            }
        }
    }

    fn new_player(&mut self, name: PlayerName) -> Result<(PlayerId, UnboundedReceiver<RoomEvent>),GameError> {

        if self.players.len() >= MAX_PLAYERS {
            return Err(GameError::TooManyPlayers);
        }

        if self.players.iter().any(|(_, player)| player.name == name ) {
            return Err(GameError::NameAlreadyExists);
        }

        let (tx_channel, rx_channel) = mpsc::unbounded_channel();
        let player_id = thread_rng().gen::<PlayerId>();

        self.players.insert(
            player_id,
            Player {
                id: player_id,
                name: name.clone(),
                tx: tx_channel,
            },
        );

        let event = RoomEvent::PlayerJoined {
            room_id: self.id,
            player_id,
            player_name: name,
        };

        self.send_all_players(event);

        Ok((player_id, rx_channel))
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
    PlayerLeft(PlayerId),
}

pub enum RoomCommand {
    AddPlayer {
        name: PlayerName,
        cmd_tx: oneshot::Sender<Result<(PlayerId, UnboundedReceiver<RoomEvent>),GameError>>,
    },
    RemovePlayer {
        id: PlayerId,
        cmd_tx: oneshot::Sender<()>,
    },
    StartGame {
        cmd_tx: oneshot::Sender<Result<(),GameError>>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_player() {
        let room_commander = RoomServer::new();
        let player_name = "name1";
        let (player_id, player_receiver) = room_commander.new_player(player_name.into()).await.unwrap();

        let received_event = get_nth_event(player_receiver, 1).await;
        assert!(
            matches!(received_event, RoomEvent::PlayerJoined { room_id: _, player_id: received_player_id, player_name: received_player_name} if received_player_id==player_id && received_player_name==player_name)
        );
    }

    #[tokio::test]
    async fn new_player_previous_player_should_receive_the_join_event() {
        let room_commander = RoomServer::new();
        let (player_name_1, player_name_2) = ("name1", "name2");
        let (_, player_receiver_1) = room_commander.new_player(player_name_1.into()).await.unwrap();
        let (player_id_2, _) = room_commander.new_player(player_name_2.into()).await.unwrap();

        let received_event = get_nth_event(player_receiver_1, 2).await;
        assert!(
            matches!(received_event, RoomEvent::PlayerJoined { room_id: _, player_id: received_player_id, player_name: received_player_name} if received_player_id==player_id_2 && received_player_name==player_name_2)
        );
    }

    #[tokio::test]
    async fn new_player_should_fail_when_name_exists() {
        let room_commander = RoomServer::new();
        let (player_name_1, player_name_2) = ("name1", "name1");
        let _ = room_commander.new_player(player_name_1.into()).await.unwrap();
        let res = room_commander.new_player(player_name_2.into()).await;

        assert!(matches!(res, Err(GameError::NameAlreadyExists)));
    }

    #[tokio::test]
    async fn new_player_should_fail_when_there_are_too_many_players() {
        let mut room_commander = RoomServer::new();
        create_n_players(&mut room_commander, 6).await;
        let res = room_commander.new_player("player6".into()).await;

        assert!(matches!(res, Err(GameError::TooManyPlayers)));
    }


    #[tokio::test]
    async fn remove_player() {
        let room_commander = RoomServer::new();
        let (player_name_1, player_name_2, player_name_3) = ("name1", "name2", "name3");
        let (_, player_receiver_1) = room_commander.new_player(player_name_1.into()).await.unwrap();
        let (_, player_receiver_2) = room_commander.new_player(player_name_2.into()).await.unwrap();
        let (player_id_3, _) = room_commander.new_player(player_name_3.into()).await.unwrap();
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

    #[tokio::test]
    async fn start_game_should_fail_when_not_enough_players() {
        let mut room_commander = RoomServer::new();
        create_n_players(&mut room_commander, 1).await;
        assert!(matches!(room_commander.start_game().await, Err(GameError::NotEnoughPlayers)));
    }

    async fn get_nth_event(mut rcv: UnboundedReceiver<RoomEvent>, nth: u8) -> RoomEvent {
        for _ in 1..nth {
            rcv.recv().await.unwrap();
        }
        rcv.recv().await.unwrap()
    }

    async fn create_n_players(room_commander: &mut RoomCommander, n: u8) -> Vec<(PlayerId, UnboundedReceiver<RoomEvent>)> {
        let mut players = vec![];
        for i in 0..n {
            players.push(room_commander.new_player(format!("name_{i}")).await.unwrap());
        }
        players
    }
}
