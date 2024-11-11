use std::{collections::HashMap, mem};

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
    DrawCard {
        player_id: PlayerId,
        cmd_tx: oneshot::Sender<Result<Card, GameError>>,
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
}

#[derive(Copy, Clone, PartialEq)]
pub enum Power {
    PeekOwnCard,
    PeekOtherCard,
    BlindSwap,
    CheckAndSwap,
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
}

pub struct Player {
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
    MiddleTurn(PlayerId, Card),
    PowerStage(PlayerId, Power),
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
                RoomCommand::RemovePlayer {
                    player_id: id,
                    cmd_tx,
                } => {
                    self.remove_player(id);
                    let _ = cmd_tx.send(());
                }
                RoomCommand::StartGame { cmd_tx } => {
                    let res = self.start_game();
                    let _ = cmd_tx.send(res);
                }
                RoomCommand::SetPlayerReady {
                    player_id: id,
                    cmd_tx,
                } => {
                    let res = self.set_player_ready(id);
                    let _ = cmd_tx.send(res);
                }
                RoomCommand::NextTurn => self.next_turn(),
                RoomCommand::DrawCard {
                    player_id: id,
                    cmd_tx,
                } => {
                    let res = self.draw_card(id);
                    let _ = cmd_tx.send(res);
                }
                RoomCommand::SwapCard {
                    player_id,
                    card_idx,
                    cmd_tx,
                } => {
                    let res = self.swap_card(player_id, card_idx);
                    let _ = cmd_tx.send(res);
                }
                RoomCommand::DiscardCard { player_id, cmd_tx } => {
                    let res = self.discard_card(player_id);
                    let _ = cmd_tx.send(res);
                }
                RoomCommand::PeekOwnCard {
                    player_id,
                    card_idx,
                    cmd_tx,
                } => {
                    let res = self.peek_own_card(player_id, card_idx);
                    let _ = cmd_tx.send(res);
                }
                RoomCommand::PeekOtherCard {
                    player_id,
                    other_player_id,
                    other_card_idx,
                    cmd_tx,
                } => {
                    let res = self.peek_other_card(player_id, other_player_id, other_card_idx);
                    let _ = cmd_tx.send(res);
                }
                RoomCommand::BlindSwap {
                    player_id,
                    card_idx,
                    other_player_id,
                    other_card_idx,
                    cmd_tx,
                } => {
                    let res = self.blind_swap(player_id, card_idx, other_player_id, other_card_idx);
                    let _ = cmd_tx.send(res);
                }
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

    fn draw_card(&mut self, player_id: PlayerId) -> Result<Card, GameError> {
        if self.state != State::StartTurn(player_id) {
            return Err(GameError::OperationNotAllowedAtCurrentState);
        }
        let card = self.deck.draw();
        self.state = State::MiddleTurn(player_id, card);

        let event = RoomEvent::CardWasDrawn(player_id);
        self.send_all_players(event);

        let event = RoomEvent::DrawnCard(card);
        self.send_to_player(player_id, event);

        Ok(card)
    }

    fn swap_card(&mut self, player_id: PlayerId, card_idx: usize) -> Result<(), GameError> {
        if let State::MiddleTurn(stored_player_id, mut card) = self.state {
            if player_id != stored_player_id {
                return Err(GameError::OperationNotAllowedAtCurrentState);
            }
            let player = self.players.get_mut(&player_id).unwrap();
            if card_idx >= player.cards.len() {
                return Err(GameError::InvalidCardIndex);
            }
            mem::swap(&mut card, &mut player.cards[card_idx]);
            self.deck.discard(card);
            let event = RoomEvent::CardSwapped(player_id, card_idx);
            self.send_all_players(event);
            let event = RoomEvent::CardDiscarded(player_id, card);
            self.send_all_players(event);
            self.next_turn();
            return Ok(());
        }
        Err(GameError::OperationNotAllowedAtCurrentState)
    }

    fn discard_card(&mut self, player_id: PlayerId) -> Result<(), GameError> {
        if let State::MiddleTurn(stored_player_id, card) = self.state {
            if player_id != stored_player_id {
                return Err(GameError::OperationNotAllowedAtCurrentState);
            }

            self.deck.discard(card);
            let event = RoomEvent::CardDiscarded(player_id, card);
            self.send_all_players(event);

            if let Some(power) = self.match_power(card) {
                let event = RoomEvent::PowerActivated(player_id, power);
                self.send_all_players(event);
                self.state = State::PowerStage(player_id, power);
            }

            self.next_turn();
            return Ok(());
        }
        Err(GameError::OperationNotAllowedAtCurrentState)
    }

    fn peek_own_card(&mut self, player_id: PlayerId, card_idx: usize) -> Result<(), GameError> {
        if let State::PowerStage(stored_player_id, Power::PeekOwnCard) = self.state {
            if player_id != stored_player_id {
                return Err(GameError::OperationNotAllowedAtCurrentState);
            }

            let player = self.players.get(&player_id).unwrap();
            if card_idx >= player.cards.len() {
                return Err(GameError::InvalidCardIndex);
            }
            let card = player.cards[card_idx];
            let event = RoomEvent::PeekedCard(card);
            self.send_to_player(player_id, event);

            let event =
                RoomEvent::PowerUsed(Power::PeekOwnCard, player_id, Some(card_idx), None, None);
            self.send_all_players(event);
            self.next_turn();
            return Ok(());
        }
        Err(GameError::OperationNotAllowedAtCurrentState)
    }
    fn peek_other_card(
        &mut self,
        player_id: PlayerId,
        other_player_id: PlayerId,
        other_card_idx: usize,
    ) -> Result<(), GameError> {
        if let State::PowerStage(stored_player_id, Power::PeekOtherCard) = self.state {
            if player_id != stored_player_id {
                return Err(GameError::OperationNotAllowedAtCurrentState);
            }

            let player = self.players.get(&other_player_id).unwrap();
            if other_card_idx >= player.cards.len() {
                return Err(GameError::InvalidCardIndex);
            }
            let card = player.cards[other_card_idx];
            let event = RoomEvent::PeekedCard(card);
            self.send_to_player(player_id, event);

            let event = RoomEvent::PowerUsed(
                Power::PeekOtherCard,
                player_id,
                None,
                Some(other_player_id),
                Some(other_card_idx),
            );
            self.send_all_players(event);
            self.next_turn();
            return Ok(());
        }
        Err(GameError::OperationNotAllowedAtCurrentState)
    }

    fn blind_swap(
        &mut self,
        player_id: PlayerId,
        card_idx: usize,
        other_player_id: PlayerId,
        other_card_idx: usize,
    ) -> Result<(), GameError> {
        if let State::PowerStage(stored_player_id, Power::BlindSwap) = self.state {
            if player_id != stored_player_id {
                return Err(GameError::OperationNotAllowedAtCurrentState);
            }

            let player1 = self.players.get(&player_id).unwrap();
            if card_idx >= player1.cards.len() {
                return Err(GameError::InvalidCardIndex);
            }

            let player2 = self.players.get(&other_player_id).unwrap();
            if other_card_idx >= player2.cards.len() {
                return Err(GameError::InvalidCardIndex);
            }

            let mut cards_2 =
                std::mem::take(&mut self.players.get_mut(&other_player_id).unwrap().cards);

            mem::swap(
                &mut self.players.get_mut(&player_id).unwrap().cards[card_idx],
                &mut cards_2[other_card_idx],
            );

            let _ = mem::replace(
                &mut self.players.get_mut(&other_player_id).unwrap().cards,
                cards_2,
            );

            let event = RoomEvent::PowerUsed(
                Power::BlindSwap,
                player_id,
                Some(card_idx),
                Some(other_player_id),
                Some(other_card_idx),
            );
            self.send_all_players(event);
            self.next_turn();
            return Ok(());
        }
        Err(GameError::OperationNotAllowedAtCurrentState)
    }

    fn match_power(&self, card: Card) -> Option<Power> {
        match card {
            Card::Clubs(n) | Card::Diamonds(n) | Card::Hearts(n) | Card::Spade(n) => match n {
                n if n < 7 => None,
                7 | 8 => Some(Power::PeekOwnCard),
                9 | 10 => Some(Power::PeekOtherCard),
                11 | 12 => Some(Power::BlindSwap),
                13 => Some(Power::CheckAndSwap),
                _ => unreachable!(),
            },
            Card::Joker => None,
        }
    }

    fn send_to_player(&self, player_id: PlayerId, event: RoomEvent) {
        let player = self.players.get(&player_id).unwrap();
        let _ = player.tx.send(event);
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

    #[tokio::test]
    async fn draw_card() {
        pause();
        let mut room_commander = RoomServer::start();
        let mut players = create_n_players(&mut room_commander, 6, true).await;
        room_commander.start_game().await.unwrap();

        clean_events(&mut players).await;
        sleep(PEEKING_PHASE_COUNTDOWN).await;
        sleep(Duration::from_secs(1)).await; //give breathing room

        if let RoomEvent::PlayerTurn(player_id) = players[0].1.recv().await.unwrap() {
            clean_events(&mut players).await;
            room_commander.draw_card(player_id).await.unwrap();
            for (_, player_rx) in players.iter_mut() {
                let received_event = get_nth_event(player_rx, 1).await;
                assert!(matches!(received_event, RoomEvent::CardWasDrawn(id) if player_id==id));
            }
            let (_, player_rx) = players.iter_mut().find(|(id, _)| *id == player_id).unwrap();
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::DrawnCard(_)));
        } else {
            panic!("Did not return PlayerTurn event")
        }
    }

    #[tokio::test]
    async fn swap_card() {
        pause();
        let mut room_commander = RoomServer::start();
        let mut players = create_n_players(&mut room_commander, 6, true).await;
        room_commander.start_game().await.unwrap();

        let peeked_cards: Vec<(Card, Card)> = players
            .iter_mut()
            .map(|(_, player)| {
                let peeked = player.try_recv().unwrap();
                if let RoomEvent::PeekingPhaseStarted((card1, card2)) = peeked {
                    (card1, card2)
                } else {
                    panic!("Did not return peeking phase event")
                }
            })
            .collect();

        clean_events(&mut players).await;
        sleep(PEEKING_PHASE_COUNTDOWN).await;
        sleep(Duration::from_secs(1)).await; //give breathing room

        if let RoomEvent::PlayerTurn(player_id) = players[0].1.recv().await.unwrap() {
            clean_events(&mut players).await;
            room_commander.draw_card(player_id).await.unwrap();
            for (_, player_rx) in players.iter_mut() {
                let received_event = get_nth_event(player_rx, 1).await;
                assert!(matches!(received_event, RoomEvent::CardWasDrawn(id) if player_id==id));
            }
            clean_events(&mut players).await;

            let (card1, _) = peeked_cards
                .get(players.iter().position(|(id, _)| *id == player_id).unwrap())
                .unwrap();
            let idx_card_to_swap = 0;
            room_commander
                .swap_card(player_id, idx_card_to_swap)
                .await
                .unwrap();
            for (_, player_rx) in players.iter_mut() {
                let received_event = get_nth_event(player_rx, 1).await;
                assert!(
                    matches!(received_event, RoomEvent::CardSwapped(id, idx_card) if player_id==id  && idx_card_to_swap == idx_card)
                );
                let received_event = get_nth_event(player_rx, 1).await;
                assert!(
                    matches!(received_event, RoomEvent::CardDiscarded(id, card) if player_id == id && card == *card1)
                );
                let received_event = get_nth_event(player_rx, 1).await;
                assert!(matches!(received_event, RoomEvent::PlayerTurn(_)));
            }
        } else {
            panic!("Did not return PlayerTurn event")
        }
    }

    #[tokio::test]
    async fn discard_card_normal_card() {
        let drawn_card = Card::Diamonds(1);
        let state = State::MiddleTurn(0, drawn_card);
        let deck = Deck::new();
        let cards = vec![
            Card::Clubs(1),
            Card::Clubs(2),
            Card::Clubs(3),
            Card::Clubs(4),
        ];

        let (room_commander, mut players_rxs) =
            init_specific_game_room(0, state, deck, cards, None);

        let _ = room_commander.discard_card(0).await;

        for player_rx in players_rxs.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(
                received_event,
                RoomEvent::CardDiscarded(id, card) if id==0 && card == drawn_card
            ));
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::PlayerTurn(1)));
        }
    }

    #[tokio::test]
    async fn discard_power() {
        let drawn_card = Card::Diamonds(7);
        let state = State::MiddleTurn(0, drawn_card);
        let deck = Deck::new();
        let cards = vec![
            Card::Clubs(1),
            Card::Clubs(2),
            Card::Clubs(3),
            Card::Clubs(4),
        ];

        let (room_commander, mut players_rxs) =
            init_specific_game_room(0, state, deck, cards, None);

        room_commander.discard_card(0).await.unwrap();

        for player_rx in players_rxs.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(
                received_event,
                RoomEvent::CardDiscarded(id, card) if id==0 && card == drawn_card
            ));
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::PowerActivated(id, _) if id==0));
        }
    }

    #[tokio::test]
    async fn use_power_peek_own_card() {
        let state = State::PowerStage(0, Power::PeekOwnCard);
        let deck = Deck::new();
        let cards = vec![
            Card::Clubs(1),
            Card::Clubs(2),
            Card::Clubs(3),
            Card::Clubs(4),
        ];

        let (room_commander, mut players_rxs) =
            init_specific_game_room(0, state, deck, cards.clone(), None);

        room_commander.peek_own_card(0, 3).await.unwrap();

        let received_event = players_rxs[0].try_recv().unwrap();
        assert!(matches!(
            received_event,
            RoomEvent::PeekedCard(card) if card==cards[3]
        ));

        for player_rx in players_rxs.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(
                received_event,
                RoomEvent::PowerUsed(Power::PeekOwnCard, 0, Some(3), None, None)
            ));
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::PlayerTurn(1)));
        }
    }

    #[tokio::test]
    async fn use_power_peek_other_card() {
        let state = State::PowerStage(1, Power::PeekOtherCard);
        let deck = Deck::new();
        let cards = vec![
            Card::Clubs(1),
            Card::Clubs(2),
            Card::Clubs(3),
            Card::Clubs(4),
        ];

        let (room_commander, mut players_rxs) =
            init_specific_game_room(1, state, deck, cards.clone(), None);

        room_commander.peek_other_card(1, 0, 3).await.unwrap();

        let received_event = players_rxs[1].try_recv().unwrap();
        assert!(matches!(
            received_event,
            RoomEvent::PeekedCard(card) if card==cards[3]
        ));

        for player_rx in players_rxs.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(
                received_event,
                RoomEvent::PowerUsed(Power::PeekOtherCard, 1, None, Some(0), Some(3))
            ));
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::PlayerTurn(2)));
        }
    }

    #[tokio::test]
    async fn use_power_blind_swap() {
        let state = State::PowerStage(0, Power::BlindSwap);
        let deck = Deck::new();
        let cards_1 = vec![
            Card::Clubs(1),
            Card::Clubs(2),
            Card::Clubs(3),
            Card::Clubs(4),
        ];
        let cards_2 = vec![
            Card::Diamonds(1),
            Card::Diamonds(2),
            Card::Diamonds(3),
            Card::Diamonds(4),
        ];

        let (room_commander, mut players_rxs) =
            init_specific_game_room(0, state, deck, cards_1.clone(), Some(cards_2.clone()));

        room_commander.blind_swap(0, 2, 1, 3).await.unwrap();

        for player_rx in players_rxs.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(
                received_event,
                RoomEvent::PowerUsed(Power::BlindSwap, 0, Some(2), Some(1), Some(3))
            ));
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::PlayerTurn(1)));
        }

        room_commander.draw_card(1).await.unwrap();
        players_rxs
            .iter_mut()
            .for_each(|rx| while rx.try_recv().is_ok() {});
        room_commander.swap_card(1, 3).await.unwrap();
        for player_rx in players_rxs.iter_mut() {
            let received_event = get_nth_event(player_rx, 2).await;
            assert!(
                matches!(received_event, RoomEvent::CardDiscarded(1, card) if card == cards_1[2])
            );
        }
    }

    // UTILS
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

    fn init_specific_game_room(
        current_player_idx: usize,
        state: State,
        deck: Deck,
        current_player_cards: Vec<Card>,
        other_player_cards: Option<Vec<Card>>,
    ) -> (RoomCommander, Vec<UnboundedReceiver<RoomEvent>>) {
        let (tx_channel, rx_channel) = mpsc::unbounded_channel();

        let mut players = HashMap::new();

        let mut players_rxs = vec![];

        let (player, player_rx) = init_specific_player(0, current_player_cards);
        players.insert(0, player);
        players_rxs.push(player_rx);

        if let Some(cards) = other_player_cards {
            let (player, player_rx) = init_specific_player(1, cards);
            players.insert(1, player);
            players_rxs.push(player_rx);
        }

        for i in players.len() as PlayerId..6 {
            let (player, player_rx) = init_specific_player(i, vec![]);
            players.insert(i, player);
            players_rxs.push(player_rx);
        }

        let room_server = RoomServer {
            id: thread_rng().gen::<RoomId>(),
            tx_channel: tx_channel.clone(),
            rx_channel,
            players,
            deck,
            state,
            current_player_idx,
            turn_order: HashMap::from([(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5)]),
        };

        spawn(room_server.run());

        (RoomCommander::new(tx_channel), players_rxs)
    }

    fn init_specific_player(
        id: PlayerId,
        cards: Vec<Card>,
    ) -> (Player, UnboundedReceiver<RoomEvent>) {
        let (tx_channel, rx_channel) = mpsc::unbounded_channel();
        (
            Player {
                name: format!("Player_{id}"),
                tx: tx_channel,
                cards,
                ready: true,
            },
            rx_channel,
        )
    }
}
