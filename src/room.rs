use std::collections::HashMap;

use rand::{thread_rng, Rng};
use tokio::{
    spawn,
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    time::sleep,
};

use crate::{
    consts::{
        GameError, PlayerId, PlayerName, RoomId, MAX_PLAYERS, MIN_PLAYERS, PEEKING_PHASE_COUNTDOWN,
    },
    deck::{Card, Deck},
    room_commander::RoomCommander,
};

pub enum RoomCommand {
    AddPlayer {
        name: PlayerName,
        cmd_tx: oneshot::Sender<Result<(PlayerId, UnboundedReceiver<RoomEvent>), GameError>>,
    },
    RemovePlayer {
        id: PlayerId,
        cmd_tx: oneshot::Sender<()>,
    },
    StartGame {
        cmd_tx: oneshot::Sender<Result<(), GameError>>,
    },
    SetPlayerReady {
        id: PlayerId,
        cmd_tx: oneshot::Sender<Result<(), GameError>>,
    },
    NextTurn,
}

#[derive(Clone)]
pub enum RoomEvent {
    PlayerJoined {
        room_id: RoomId,
        player_id: PlayerId,
        player_name: PlayerName,
    },
    PlayerLeft(PlayerId),
    GameStarted,
    PlayerTurn(PlayerId),
    PeekingPhaseStarted((Card, Card)),
    PlayerIsReady(PlayerId),
}

pub struct Player {
    id: PlayerId,
    name: PlayerName,
    tx: UnboundedSender<RoomEvent>,
    cards: Vec<Card>,
    ready: bool,
}

#[derive(PartialEq)]
pub enum State {
    NotStarted,
    PeekingPhase,
    StartTurn(PlayerId),
}
pub struct RoomServer {
    id: RoomId,
    tx_channel: UnboundedSender<RoomCommand>,
    rx_channel: UnboundedReceiver<RoomCommand>,
    players: HashMap<PlayerId, Player>,
    deck: Deck,
    state: State,
    current_player_idx: usize,
    turn_order: HashMap<usize, PlayerId>,
}

impl RoomServer {
    pub fn start() -> RoomCommander {
        let (tx_channel, rx_channel) = mpsc::unbounded_channel();

        let room_server = Self {
            id: thread_rng().gen::<RoomId>(),
            tx_channel: tx_channel.clone(),
            rx_channel,
            players: HashMap::with_capacity(6),
            deck: Deck::new(),
            state: State::NotStarted,
            current_player_idx: 0,
            turn_order: HashMap::with_capacity(6),
        };

        spawn(room_server.run());

        RoomCommander::new(tx_channel)
    }
    async fn run(mut self) {
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
                    let res = self.start_game();
                    let _ = cmd_tx.send(res);
                }
                RoomCommand::SetPlayerReady { id, cmd_tx } => {
                    let res = self.set_player_ready(id);
                    let _ = cmd_tx.send(res);
                }
                RoomCommand::NextTurn => self.next_turn(),
            }
        }
    }

    fn start_game(&mut self) -> Result<(), GameError> {
        if self.state != State::NotStarted {
            return Err(GameError::OperationNotAllowedAtCurrentState);
        }

        if self.players.len() < MIN_PLAYERS {
            return Err(GameError::NotEnoughPlayers);
        }

        for (i, (&player_id, _)) in self.players.iter().enumerate() {
            self.turn_order.insert(i, player_id);
        }

        self.state = State::PeekingPhase;

        self.deal_cards_and_peek();

        Ok(())
    }

    fn deal_cards_and_peek(&mut self) {
        self.players.iter_mut().for_each(|(_, player)| {
            for _ in 0..4 {
                player.cards.push(self.deck.draw());
            }
            let _ = player.tx.send(RoomEvent::PeekingPhaseStarted((
                player.cards[0],
                player.cards[1],
            )));
        });
        spawn(Self::peeking_phase_countdown(self.tx_channel.clone()));
    }

    fn new_player(
        &mut self,
        name: PlayerName,
    ) -> Result<(PlayerId, UnboundedReceiver<RoomEvent>), GameError> {
        if self.state != State::NotStarted {
            return Err(GameError::OperationNotAllowedAtCurrentState);
        }

        if self.players.len() >= MAX_PLAYERS {
            return Err(GameError::TooManyPlayers);
        }

        if self.players.iter().any(|(_, player)| player.name == name) {
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
                cards: vec![],
                ready: false,
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

    fn set_player_ready(&mut self, id: PlayerId) -> Result<(), GameError> {
        if self.state != State::PeekingPhase {
            return Err(GameError::OperationNotAllowedAtCurrentState);
        }

        let player = self.players.get_mut(&id).unwrap();
        player.ready = true;
        let event = RoomEvent::PlayerIsReady(id);
        self.send_all_players(event);
        if self.players.iter().all(|(_, player)| player.ready) {
            let _ = self.tx_channel.send(RoomCommand::NextTurn);
        }
        Ok(())
    }

    fn next_turn(&mut self) {
        self.current_player_idx += 1;
        let idx = self.current_player_idx % self.players.len();
        let current_player_id = self.turn_order[&idx];
        self.state = State::StartTurn(current_player_id);

        let event = RoomEvent::PlayerTurn(current_player_id);
        self.send_all_players(event);
    }

    fn send_all_players(&self, event: RoomEvent) {
        self.players.iter().for_each(|(_, player)| {
            let _ = player.tx.send(event.clone());
        });
    }

    async fn peeking_phase_countdown(tx_channel: UnboundedSender<RoomCommand>) {
        sleep(PEEKING_PHASE_COUNTDOWN).await;
        let _ = tx_channel.send(RoomCommand::NextTurn);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::time::pause;

    use crate::room_commander::RoomCommander;

    use super::*;

    #[tokio::test]
    async fn new_player() {
        let mut room_commander = RoomServer::start();
        let mut players = create_n_players(&mut room_commander, 1, false).await;

        let received_event = get_nth_event(&mut players[0].1, 1).await;
        assert!(
            matches!(received_event, RoomEvent::PlayerJoined { room_id: _, player_id: received_player_id, player_name: received_player_name} if received_player_id==players[0].0 && received_player_name=="name_0")
        );
    }

    #[tokio::test]
    async fn new_player_previous_player_should_receive_the_join_event() {
        let mut room_commander = RoomServer::start();
        let mut players = create_n_players(&mut room_commander, 2, false).await;

        let received_event = get_nth_event(&mut players[0].1, 2).await;
        assert!(
            matches!(received_event, RoomEvent::PlayerJoined { room_id: _, player_id: received_player_id, player_name: received_player_name} if received_player_id==players[1].0 && received_player_name=="name_1")
        );
    }

    #[tokio::test]
    async fn new_player_should_fail_when_name_exists() {
        let room_commander = RoomServer::start();
        let (player_name_1, player_name_2) = ("name1", "name1");
        let _ = room_commander
            .new_player(player_name_1.into())
            .await
            .unwrap();
        let res = room_commander.new_player(player_name_2.into()).await;

        assert!(matches!(res, Err(GameError::NameAlreadyExists)));
    }

    #[tokio::test]
    async fn new_player_should_fail_when_there_are_too_many_players() {
        let mut room_commander = RoomServer::start();
        create_n_players(&mut room_commander, 6, false).await;
        let res = room_commander.new_player("name_7".into()).await;

        assert!(matches!(res, Err(GameError::TooManyPlayers)));
    }

    #[tokio::test]
    async fn remove_player() {
        let mut room_commander: RoomCommander = RoomServer::start();
        let mut players = create_n_players(&mut room_commander, 3, false).await;
        room_commander.remove_player(players[2].0).await;

        let received_event = get_nth_event(&mut players[0].1, 4).await;
        assert!(
            matches!(received_event, RoomEvent::PlayerLeft(received_player_id) if received_player_id == players[2].0)
        );

        let received_event = get_nth_event(&mut players[1].1, 3).await;
        assert!(
            matches!(received_event, RoomEvent::PlayerLeft(received_player_id) if received_player_id == players[2].0)
        );
    }

    #[tokio::test]
    async fn start_game_should_fail_when_not_enough_players() {
        let mut room_commander = RoomServer::start();
        create_n_players(&mut room_commander, 1, false).await;
        assert!(matches!(
            room_commander.start_game().await,
            Err(GameError::NotEnoughPlayers)
        ));
    }

    #[tokio::test]
    async fn start_game() {
        let mut room_commander = RoomServer::start();
        let mut players = create_n_players(&mut room_commander, 6, true).await;
        room_commander.start_game().await.unwrap();

        for (_, player_rx) in players.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(
                received_event,
                RoomEvent::PeekingPhaseStarted((_, _))
            ));
        }
    }

    #[tokio::test]
    async fn cannot_start_game_if_state_different_from_not_started() {
        let mut room_commander = RoomServer::start();
        create_n_players(&mut room_commander, 6, true).await;
        room_commander.start_game().await.unwrap();
        assert!(matches!(
            room_commander.start_game().await,
            Err(GameError::OperationNotAllowedAtCurrentState)
        ));
    }

    #[tokio::test]
    async fn new_player_should_fail_when_game_started() {
        let mut room_commander = RoomServer::start();
        create_n_players(&mut room_commander, 6, true).await;
        room_commander.start_game().await.unwrap();

        assert!(matches!(
            room_commander.new_player("test".into()).await,
            Err(GameError::OperationNotAllowedAtCurrentState)
        ));
    }

    #[tokio::test]
    async fn start_turn_when_everyone_is_ready() {
        let mut room_commander = RoomServer::start();
        let mut players = create_n_players(&mut room_commander, 6, true).await;
        room_commander.start_game().await.unwrap();

        clean_events(&mut players).await;

        for (player_id, _) in players.iter() {
            let _ = room_commander.set_player_ready(*player_id).await;
        }

        for (_, player_rx) in players.iter_mut() {
            let received_event = get_nth_event(player_rx, 6).await;
            assert!(matches!(received_event, RoomEvent::PlayerIsReady(_)));
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::PlayerTurn(_)));
        }
    }

    #[tokio::test]
    async fn cannot_set_ready_when_stage_is_not_peeking_phase() {
        pause();
        let mut room_commander = RoomServer::start();
        let mut players = create_n_players(&mut room_commander, 6, true).await;
        room_commander.start_game().await.unwrap();

        clean_events(&mut players).await;
        sleep(PEEKING_PHASE_COUNTDOWN).await;
        sleep(Duration::from_secs(1)).await; //give breathing room
        clean_events(&mut players).await;

        assert!(matches!(
            room_commander.set_player_ready(players[0].0).await,
            Err(GameError::OperationNotAllowedAtCurrentState)
        ));
    }

    #[tokio::test]
    async fn automatic_start_turn_after_timeout() {
        pause();
        let mut room_commander = RoomServer::start();
        let mut players = create_n_players(&mut room_commander, 6, true).await;
        room_commander.start_game().await.unwrap();

        clean_events(&mut players).await;
        sleep(PEEKING_PHASE_COUNTDOWN).await;
        sleep(Duration::from_secs(1)).await; //give breathing room

        for (_, player_rx) in players.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::PlayerTurn(_)));
        }
    }

    async fn get_nth_event(rcv: &mut UnboundedReceiver<RoomEvent>, nth: u8) -> RoomEvent {
        for _ in 1..nth {
            rcv.try_recv().unwrap();
        }
        rcv.try_recv().unwrap()
    }

    async fn create_n_players(
        room_commander: &mut RoomCommander,
        n: u8,
        consume_events: bool,
    ) -> Vec<(PlayerId, UnboundedReceiver<RoomEvent>)> {
        let mut players = vec![];
        for i in 0..n {
            players.push(
                room_commander
                    .new_player(format!("name_{i}"))
                    .await
                    .unwrap(),
            );
        }
        if consume_events {
            clean_events(&mut players).await;
        }
        players
    }
    async fn clean_events(players: &mut [(PlayerId, UnboundedReceiver<RoomEvent>)]) {
        players
            .iter_mut()
            .for_each(|(_, rx)| while rx.try_recv().is_ok() {});
    }
}
