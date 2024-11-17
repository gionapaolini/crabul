use std::{collections::HashMap, mem};

use rand::{seq::IteratorRandom, thread_rng, Rng};
use serde::Serialize;
use tokio::{
    spawn,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
    time::sleep,
};

use crate::{
    consts::{PlayerId, PlayerName, RoomId},
    deck::{Card, Deck},
    room::{commander::RoomCommander, commands::RoomCommand, events::RoomEvent},
};

use super::{
    consts::{
        FINALIZE_GAME_COUNTDOWN, MAX_PLAYERS, MIN_PLAYERS, PEEKING_PHASE_COUNTDOWN, TURN_COUNTDOWN,
    },
    errors::GameError,
};

#[derive(Serialize, Copy, Clone, PartialEq)]
pub enum Power {
    PeekOwnCard,
    PeekOtherCard,
    BlindSwap,
    CheckAndSwapStage1,
    CheckAndSwapStage2(PlayerId, usize),
}

#[derive(Serialize, Clone)]
pub enum SameCardResult {
    Success,
    NotTheSame,
    TooLate,
}

#[derive(Serialize, Clone, PartialEq)]
pub struct Score {
    player_id: PlayerId,
    cards: Vec<Card>,
    total_score: i8,
}
#[derive(Serialize, Clone)]
pub struct FinalScore {
    pub winner: PlayerId,
    pub scores: Vec<Score>,
}

pub struct Player {
    name: PlayerName,
    tx: UnboundedSender<RoomEvent>,
    cards: Vec<Card>,
    ready: bool,
}

#[derive(PartialEq, Clone)]
pub enum State {
    NotStarted,
    PeekingPhase,
    StartTurn(PlayerId),
    MiddleTurn(PlayerId, Card),
    PowerStage(PlayerId, Power),
    PauseForSameCardThrow(PlayerId, PlayerId, usize, Box<State>),
    Terminating,
    Terminated,
}
pub struct RoomServer {
    id: RoomId,
    tx_channel: UnboundedSender<RoomCommand>,
    rx_channel: UnboundedReceiver<RoomCommand>,
    players: HashMap<PlayerId, Player>,
    deck: Deck,
    state: State,
    same_card_thrown: bool,
    current_player_idx: usize,
    turn_order: HashMap<usize, PlayerId>,
    crabul_player: Option<PlayerId>,
    current_count_down: Option<JoinHandle<()>>,
}

impl RoomServer {
    pub fn new() -> (Self, RoomCommander) {
        let (tx_channel, rx_channel) = mpsc::unbounded_channel();

        let room_server = Self {
            id: thread_rng().gen::<RoomId>(),
            tx_channel: tx_channel.clone(),
            rx_channel,
            players: HashMap::with_capacity(6),
            deck: Deck::new(),
            state: State::NotStarted,
            same_card_thrown: false,
            current_player_idx: 0,
            turn_order: HashMap::with_capacity(6),
            crabul_player: None,
            current_count_down: None,
        };

        (room_server, RoomCommander::new(tx_channel))
    }

    pub fn start() -> (RoomId, RoomCommander) {
        let (tx_channel, rx_channel) = mpsc::unbounded_channel();

        let id = thread_rng().gen::<RoomId>();
        let room_server = Self {
            id,
            tx_channel: tx_channel.clone(),
            rx_channel,
            players: HashMap::with_capacity(6),
            deck: Deck::new(),
            state: State::NotStarted,
            same_card_thrown: false,
            current_player_idx: 0,
            turn_order: HashMap::with_capacity(6),
            crabul_player: None,
            current_count_down: None,
        };

        spawn(room_server.run());

        (id, RoomCommander::new(tx_channel))
    }

    async fn run(mut self) {
        while let Some(cmd) = self.rx_channel.recv().await {
            match cmd {
                RoomCommand::AddPlayer { name, cmd_tx } => {
                    let res = self.new_player(name);
                    let _ = cmd_tx.send(res);
                }
                RoomCommand::RemovePlayer { player_id, cmd_tx } => {
                    self.remove_player(player_id);
                    let _ = cmd_tx.send(());
                }
                RoomCommand::StartGame { cmd_tx } => {
                    let res = self.start_game();
                    let _ = cmd_tx.send(res);
                }
                RoomCommand::SetPlayerReady { player_id, cmd_tx } => {
                    let res = self.set_player_ready(player_id);
                    let _ = cmd_tx.send(res);
                }
                RoomCommand::NextTurn => self.next_turn(),
                RoomCommand::GoCrabul { player_id, cmd_tx } => {
                    let res = self.go_crabul(player_id);
                    let _ = cmd_tx.send(res);
                }
                RoomCommand::DrawCard { player_id, cmd_tx } => {
                    let res = self.draw_card(player_id);
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
                RoomCommand::CheckAndSwapStage1 {
                    player_id,
                    other_player_id,
                    other_card_idx,
                    cmd_tx,
                } => {
                    let res =
                        self.check_and_swap_stage1(player_id, other_player_id, other_card_idx);
                    let _ = cmd_tx.send(res);
                }
                RoomCommand::CheckAndSwapStage2 {
                    player_id,
                    card_idx,
                    cmd_tx,
                } => {
                    let res = self.check_and_swap_stage2(player_id, card_idx);
                    let _ = cmd_tx.send(res);
                }
                RoomCommand::ThrowSameCard {
                    player_id,
                    picked_player_id,
                    picked_card_idx,
                    cmd_tx,
                } => {
                    let res = self.throw_same_card(player_id, picked_player_id, picked_card_idx);
                    let _ = cmd_tx.send(res);
                }
                RoomCommand::SelectCardToGiveAway {
                    player_id,
                    card_idx,
                    cmd_tx,
                } => {
                    let res = self.select_card_to_give_away(player_id, card_idx);
                    let _ = cmd_tx.send(res);
                }
                RoomCommand::StopRoomServer => break,
                RoomCommand::ForceEndTurn(player_id) => {
                    self.force_end_turn(player_id);
                }
                RoomCommand::FinalizeGame => {
                    self.finalize_game();
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

        if name.is_empty() {
            return Err(GameError::EmptyName);
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
            player_list: self
                .players
                .iter()
                .map(|(id, player)| (*id, player.name.clone()))
                .collect(),
        };

        self.send_all_players(event);

        Ok((player_id, rx_channel))
    }

    fn remove_player(&mut self, id: PlayerId) {
        self.players.remove(&id);
        if self.players.is_empty() {
            let _ = self.tx_channel.send(RoomCommand::StopRoomServer);
            return;
        }
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

    fn go_crabul(&mut self, player_id: PlayerId) -> Result<(), GameError> {
        if self.state != State::StartTurn(player_id) || self.crabul_player.is_some() {
            return Err(GameError::OperationNotAllowedAtCurrentState);
        }
        self.crabul_player = Some(player_id);
        let event = RoomEvent::PlayerWentCrabul(player_id);
        self.send_all_players(event);
        self.next_turn();
        Ok(())
    }

    fn next_turn(&mut self) {
        if let Some(current_count_down) = self.current_count_down.take() {
            current_count_down.abort();
        }
        self.same_card_thrown = false;
        self.current_player_idx += 1;
        self.current_player_idx %= self.players.len();
        let current_player_id = self.turn_order[&self.current_player_idx];

        if let Some(crabul_player) = self.crabul_player {
            if current_player_id == crabul_player {
                spawn(Self::finalize_game_countdown(self.tx_channel.clone()));
                self.state = State::Terminating;
                return;
            }
        }

        self.state = State::StartTurn(current_player_id);

        let event = RoomEvent::PlayerTurn(current_player_id);
        self.send_all_players(event);

        let count_down = spawn(Self::turn_countdown(
            current_player_id,
            self.tx_channel.clone(),
        ));

        self.current_count_down = Some(count_down);
    }

    fn finalize_game(&mut self) {
        let scores = self.players.iter().map(|(player_id, player)| Score {
            player_id: *player_id,
            cards: player.cards.clone(),
            total_score: player.cards.iter().map(|card| card.get_score()).sum(),
        });
        self.state = State::Terminated;

        let mut sorted_scores: Vec<Score> = scores.collect();
        sorted_scores.sort_by(|a, b| a.total_score.cmp(&b.total_score));
        let (winner1, winner2) = (sorted_scores[0].clone(), sorted_scores[1].clone());
        let mut final_winner = winner1.clone();

        if winner1 == winner2 && winner1.player_id == self.crabul_player.unwrap() {
            final_winner = winner2
        }

        let event = RoomEvent::GameTerminated(FinalScore {
            winner: final_winner.player_id,
            scores: sorted_scores,
        });
        self.send_all_players(event);
        let _ = self.tx_channel.send(RoomCommand::StopRoomServer);
    }

    fn draw_card(&mut self, player_id: PlayerId) -> Result<(), GameError> {
        if self.state != State::StartTurn(player_id) {
            return Err(GameError::OperationNotAllowedAtCurrentState);
        }
        let card = self.deck.draw();
        self.state = State::MiddleTurn(player_id, card);

        let event = RoomEvent::CardWasDrawn(player_id);
        self.send_all_players(event);

        let event = RoomEvent::DrawnCard(card);
        self.send_to_player(player_id, event);

        Ok(())
    }

    fn swap_card(&mut self, player_id: PlayerId, card_idx: usize) -> Result<(), GameError> {
        if let State::MiddleTurn(stored_player_id, mut card) = self.state {
            self.validate_player_turn(player_id, stored_player_id)?;
            self.validate_idx_card(player_id, card_idx)?;

            let player = self.players.get_mut(&player_id).unwrap();
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
            self.validate_player_turn(player_id, stored_player_id)?;

            self.deck.discard(card);
            let event = RoomEvent::CardDiscarded(player_id, card);
            self.send_all_players(event);

            if let Some(power) = self.match_power(card) {
                if self.crabul_player.is_some() && self.players.len() == 2 {
                    let event = RoomEvent::PowerDiscarded(player_id, power);
                    self.send_all_players(event);
                    self.next_turn();
                    return Ok(());
                }
                let event = RoomEvent::PowerActivated(player_id, power);
                self.send_all_players(event);
                self.state = State::PowerStage(player_id, power);
                return Ok(());
            }

            self.next_turn();
            return Ok(());
        }
        Err(GameError::OperationNotAllowedAtCurrentState)
    }

    fn peek_own_card(&mut self, player_id: PlayerId, card_idx: usize) -> Result<(), GameError> {
        if let State::PowerStage(stored_player_id, Power::PeekOwnCard) = self.state {
            self.validate_player_turn(player_id, stored_player_id)?;

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
            self.validate_player_turn(player_id, stored_player_id)?;
            self.validate_crabul_player(other_player_id)?;

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
            self.validate_player_turn(player_id, stored_player_id)?;
            self.validate_crabul_player(other_player_id)?;

            self.swap_players_card(player_id, card_idx, other_player_id, other_card_idx)?;

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

    fn check_and_swap_stage1(
        &mut self,
        player_id: PlayerId,
        other_player_id: PlayerId,
        other_card_idx: usize,
    ) -> Result<(), GameError> {
        if let State::PowerStage(stored_player_id, Power::CheckAndSwapStage1) = self.state {
            self.validate_player_turn(player_id, stored_player_id)?;
            self.validate_crabul_player(other_player_id)?;

            let player = self.players.get(&other_player_id).unwrap();
            if other_card_idx >= player.cards.len() {
                return Err(GameError::InvalidCardIndex);
            }
            let card = player.cards[other_card_idx];
            let event = RoomEvent::PeekedCard(card);
            self.send_to_player(player_id, event);

            let event = RoomEvent::PowerUsed(
                Power::CheckAndSwapStage1,
                player_id,
                None,
                Some(other_player_id),
                Some(other_card_idx),
            );
            self.send_all_players(event);
            self.state = State::PowerStage(
                stored_player_id,
                Power::CheckAndSwapStage2(other_player_id, other_card_idx),
            );
            return Ok(());
        }
        Err(GameError::OperationNotAllowedAtCurrentState)
    }

    fn check_and_swap_stage2(
        &mut self,
        player_id: PlayerId,
        card_idx: Option<usize>,
    ) -> Result<(), GameError> {
        if let State::PowerStage(
            stored_player_id,
            Power::CheckAndSwapStage2(other_player_id, other_card_idx),
        ) = self.state
        {
            self.validate_player_turn(player_id, stored_player_id)?;

            if let Some(card_idx) = card_idx {
                self.swap_players_card(player_id, card_idx, other_player_id, other_card_idx)?;
            }

            let event = RoomEvent::PowerUsed(
                Power::CheckAndSwapStage2(other_player_id, other_card_idx),
                player_id,
                card_idx,
                Some(other_player_id),
                Some(other_card_idx),
            );
            self.send_all_players(event);
            self.next_turn();
            return Ok(());
        }
        Err(GameError::OperationNotAllowedAtCurrentState)
    }

    fn throw_same_card(
        &mut self,
        player_id: PlayerId,
        picked_player_id: PlayerId,
        picked_card_idx: usize,
    ) -> Result<(), GameError> {
        match self.state {
            State::NotStarted | State::PeekingPhase | State::Terminated => {
                Err(GameError::OperationNotAllowedAtCurrentState)
            }
            State::StartTurn(_)
            | State::MiddleTurn(_, _)
            | State::PowerStage(_, _)
            | State::PauseForSameCardThrow(_, _, _, _)
            | State::Terminating => {
                if self.same_card_thrown {
                    self.give_penalty(
                        player_id,
                        picked_player_id,
                        picked_card_idx,
                        None,
                        SameCardResult::TooLate,
                    );
                    return Ok(());
                }
                let chosen_card = self.players[&picked_player_id].cards[picked_card_idx];
                if let Some(discarded_card) = self.deck.get_last_discarded() {
                    self.validate_idx_card(picked_player_id, picked_card_idx)?;

                    if chosen_card.get_value() == discarded_card.get_value() {
                        let card = self
                            .players
                            .get_mut(&picked_player_id)
                            .unwrap()
                            .cards
                            .remove(picked_card_idx);
                        self.deck.discard(card);
                        let event = RoomEvent::SameCardAttempt(
                            player_id,
                            picked_player_id,
                            picked_card_idx,
                            Some(card),
                            SameCardResult::Success,
                        );
                        self.send_all_players(event);
                        self.same_card_thrown = true;
                        if player_id != picked_player_id {
                            self.state = State::PauseForSameCardThrow(
                                player_id,
                                picked_player_id,
                                picked_card_idx,
                                Box::new(self.state.clone()),
                            )
                        }
                    } else {
                        self.give_penalty(
                            player_id,
                            picked_player_id,
                            picked_card_idx,
                            Some(chosen_card),
                            SameCardResult::NotTheSame,
                        );
                    }
                } else {
                    self.give_penalty(
                        player_id,
                        picked_player_id,
                        picked_card_idx,
                        Some(chosen_card),
                        SameCardResult::NotTheSame,
                    );
                }

                Ok(())
            }
        }
    }

    fn select_card_to_give_away(
        &mut self,
        player_id: PlayerId,
        card_idx: usize,
    ) -> Result<(), GameError> {
        if let State::PauseForSameCardThrow(
            stored_player_id,
            other_player_id,
            other_card_idx,
            state,
        ) = self.state.clone()
        {
            if player_id != stored_player_id {
                return Err(GameError::OperationNotAllowedAtCurrentState);
            }

            self.validate_idx_card(stored_player_id, card_idx)?;

            let card = self
                .players
                .get_mut(&player_id)
                .unwrap()
                .cards
                .remove(card_idx);
            self.players
                .get_mut(&other_player_id)
                .unwrap()
                .cards
                .insert(other_card_idx, card);

            let event =
                RoomEvent::CardReplaced(player_id, card_idx, other_player_id, other_card_idx);
            self.send_all_players(event);
            self.state = *state;
            return Ok(());
        }
        Err(GameError::OperationNotAllowedAtCurrentState)
    }

    fn give_penalty(
        &mut self,
        player_id: u16,
        picked_player_id: u16,
        picked_card_idx: usize,
        chosen_card: Option<Card>,
        result: SameCardResult,
    ) {
        let event = RoomEvent::SameCardAttempt(
            player_id,
            picked_player_id,
            picked_card_idx,
            chosen_card,
            result,
        );
        self.send_all_players(event);
        let new_card = self.deck.draw();
        self.players
            .get_mut(&player_id)
            .unwrap()
            .cards
            .push(new_card);
    }

    fn force_end_turn(&mut self, player_id: PlayerId) {
        if self.turn_order[&self.current_player_idx] != player_id {
            return;
        }
        match self.state {
            State::NotStarted | State::PeekingPhase | State::Terminating | State::Terminated => {}
            State::StartTurn(_) => {
                let event = RoomEvent::TurnEndedByTimeout(player_id);
                self.send_all_players(event);
                let card = self.deck.draw();
                let event = RoomEvent::CardWasDrawn(player_id);
                self.send_all_players(event);
                self.auto_discard_drawn_card(card, player_id);
            }
            State::MiddleTurn(_, card) => {
                let event = RoomEvent::TurnEndedByTimeout(player_id);
                self.send_all_players(event);
                self.auto_discard_drawn_card(card, player_id);
            }
            State::PowerStage(_, power) => {
                let event = RoomEvent::TurnEndedByTimeout(player_id);
                self.send_all_players(event);
                self.discard_power(player_id, power);
                self.next_turn();
            }
            State::PauseForSameCardThrow(_, _, _, _) => {
                //reset timer;
                spawn(Self::turn_countdown(player_id, self.tx_channel.clone()));
            }
        }
    }

    fn discard_power(&mut self, player_id: PlayerId, power: Power) {
        match power {
            Power::PeekOwnCard
            | Power::PeekOtherCard
            | Power::CheckAndSwapStage1
            | Power::CheckAndSwapStage2(..) => {
                let event = RoomEvent::PowerDiscarded(player_id, power);
                self.send_all_players(event);
            }
            Power::BlindSwap => {
                let rng = &mut rand::thread_rng();
                let player_list = self.players.iter().filter(|(id, _)| {
                    **id != player_id
                        && (self.crabul_player.is_none() || **id != self.crabul_player.unwrap())
                });

                if player_list.clone().count() == 0 {
                    let event = RoomEvent::PowerDiscarded(player_id, power);
                    self.send_all_players(event);
                    return;
                }

                let (other_player_id, other_player) = player_list.choose(rng).unwrap();
                let card_idx = rng.gen_range(0..self.players[&player_id].cards.len());
                let other_card_idx = rng.gen_range(0..other_player.cards.len());

                let other_player_id = *other_player_id;

                self.swap_players_card(player_id, card_idx, other_player_id, other_card_idx)
                    .unwrap();

                let event = RoomEvent::ForcedBlindSwap(
                    player_id,
                    card_idx,
                    other_player_id,
                    other_card_idx,
                );
                self.send_all_players(event);
            }
        }
    }

    fn auto_discard_drawn_card(&mut self, card: Card, player_id: u16) {
        self.deck.discard(card);
        let event = RoomEvent::CardDiscarded(player_id, card);
        self.send_all_players(event);

        if let Some(power) = self.match_power(card) {
            self.discard_power(player_id, power);
        }

        self.next_turn();
    }

    fn swap_players_card(
        &mut self,
        player_id_1: PlayerId,
        card_idx_1: usize,
        player_id_2: PlayerId,
        card_idx_2: usize,
    ) -> Result<(), GameError> {
        self.validate_idx_card(player_id_1, card_idx_1)?;
        self.validate_idx_card(player_id_2, card_idx_2)?;

        let mut cards_2 = std::mem::take(&mut self.players.get_mut(&player_id_2).unwrap().cards);

        mem::swap(
            &mut self.players.get_mut(&player_id_1).unwrap().cards[card_idx_1],
            &mut cards_2[card_idx_2],
        );

        let _ = mem::replace(
            &mut self.players.get_mut(&player_id_2).unwrap().cards,
            cards_2,
        );
        Ok(())
    }

    fn validate_idx_card(&self, player_id: PlayerId, card_idx: usize) -> Result<(), GameError> {
        let player = self.players.get(&player_id).unwrap();
        if card_idx >= player.cards.len() {
            return Err(GameError::InvalidCardIndex);
        }
        Ok(())
    }

    fn validate_player_turn(
        &self,
        player_id: PlayerId,
        stored_player_id: PlayerId,
    ) -> Result<(), GameError> {
        if player_id != stored_player_id {
            return Err(GameError::OperationNotAllowedAtCurrentState);
        }
        Ok(())
    }

    fn validate_crabul_player(&self, other_player_id: PlayerId) -> Result<(), GameError> {
        if let Some(crabul_player) = self.crabul_player {
            if crabul_player == other_player_id {
                return Err(GameError::OperationNotAllowedAtCurrentState);
            }
        }
        Ok(())
    }

    fn match_power(&self, card: Card) -> Option<Power> {
        match card {
            Card::Clubs(n) | Card::Diamonds(n) | Card::Hearts(n) | Card::Spade(n) => match n {
                n if n < 7 => None,
                7 | 8 => Some(Power::PeekOwnCard),
                9 | 10 => Some(Power::PeekOtherCard),
                11 | 12 => Some(Power::BlindSwap),
                13 => Some(Power::CheckAndSwapStage1),
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

    async fn turn_countdown(player_id: PlayerId, tx_channel: UnboundedSender<RoomCommand>) {
        sleep(TURN_COUNTDOWN).await;
        let _ = tx_channel.send(RoomCommand::ForceEndTurn(player_id));
    }

    async fn finalize_game_countdown(tx_channel: UnboundedSender<RoomCommand>) {
        sleep(FINALIZE_GAME_COUNTDOWN).await;
        let _ = tx_channel.send(RoomCommand::FinalizeGame);
    }
}

#[cfg(test)]
mod tests {
    use std::{ops::Add, time::Duration};

    use tokio::time::pause;

    use crate::deck;

    use super::*;

    #[tokio::test]
    async fn new_player() {
        let (_, mut room_commander) = RoomServer::start();
        let mut players = create_n_players(&mut room_commander, 1, false).await;

        let received_event = get_nth_event(&mut players[0].1, 1).await;

        if let RoomEvent::PlayerJoined {
            room_id: _,
            player_id,
            player_name,
            player_list,
        } = received_event
        {
            assert!(player_id == players[0].0);
            assert!(player_name == "name_0");
            assert!(player_list == HashMap::from([(players[0].0, "name_0".into())]));
        }
    }

    #[tokio::test]
    async fn new_player_previous_player_should_receive_the_join_event() {
        let (_, mut room_commander) = RoomServer::start();
        let mut players = create_n_players(&mut room_commander, 2, false).await;

        let received_event = get_nth_event(&mut players[0].1, 2).await;
        if let RoomEvent::PlayerJoined {
            room_id: _,
            player_id,
            player_name,
            player_list,
        } = received_event
        {
            assert!(player_id == players[1].0);
            assert!(player_name == "name_1");
            assert!(
                player_list
                    == HashMap::from([
                        (players[0].0, "name_0".into()),
                        (players[1].0, "name_1".into())
                    ])
            );
        }
    }

    #[tokio::test]
    async fn new_player_should_fail_when_name_exists() {
        let (_, room_commander) = RoomServer::start();
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
        let (_, mut room_commander) = RoomServer::start();
        create_n_players(&mut room_commander, 6, false).await;
        let res = room_commander.new_player("name_7".into()).await;

        assert!(matches!(res, Err(GameError::TooManyPlayers)));
    }

    #[tokio::test]
    async fn remove_player() {
        let (_, mut room_commander) = RoomServer::start();
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
        let (_, mut room_commander) = RoomServer::start();
        create_n_players(&mut room_commander, 1, false).await;
        assert!(matches!(
            room_commander.start_game().await,
            Err(GameError::NotEnoughPlayers)
        ));
    }

    #[tokio::test]
    async fn start_game() {
        let (_, mut room_commander) = RoomServer::start();
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
        let (_, mut room_commander) = RoomServer::start();
        create_n_players(&mut room_commander, 6, true).await;
        room_commander.start_game().await.unwrap();
        assert!(matches!(
            room_commander.start_game().await,
            Err(GameError::OperationNotAllowedAtCurrentState)
        ));
    }

    #[tokio::test]
    async fn new_player_should_fail_when_game_started() {
        let (_, mut room_commander) = RoomServer::start();
        create_n_players(&mut room_commander, 6, true).await;
        room_commander.start_game().await.unwrap();

        assert!(matches!(
            room_commander.new_player("test".into()).await,
            Err(GameError::OperationNotAllowedAtCurrentState)
        ));
    }

    #[tokio::test]
    async fn start_turn_when_everyone_is_ready() {
        let (_, mut room_commander) = RoomServer::start();
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
        let (_, mut room_commander) = RoomServer::start();
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
        let (_, mut room_commander) = RoomServer::start();
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
        let (_, mut room_commander) = RoomServer::start();
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
        let (_, mut room_commander) = RoomServer::start();
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

    #[tokio::test]
    async fn use_power_check_and_swap_stage1() {
        let state = State::PowerStage(0, Power::CheckAndSwapStage1);
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

        room_commander.check_and_swap_stage1(0, 1, 3).await.unwrap();

        let received_event = players_rxs[0].try_recv().unwrap();
        assert!(matches!(
            received_event,
            RoomEvent::PeekedCard(card) if card==cards_2[3]
        ));

        for player_rx in players_rxs.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(
                received_event,
                RoomEvent::PowerUsed(Power::CheckAndSwapStage1, 0, None, Some(1), Some(3))
            ));
        }
    }

    #[tokio::test]
    async fn use_power_check_and_swap_decide_to_swap() {
        let state = State::PowerStage(0, Power::CheckAndSwapStage2(1, 3));
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

        room_commander
            .check_and_swap_stage2(0, Some(2))
            .await
            .unwrap();

        for player_rx in players_rxs.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(
                received_event,
                RoomEvent::PowerUsed(
                    Power::CheckAndSwapStage2(_, _),
                    0,
                    Some(2),
                    Some(1),
                    Some(3)
                )
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

    #[tokio::test]
    async fn use_power_check_and_swap_decide_to_not_swap() {
        let state = State::PowerStage(0, Power::CheckAndSwapStage2(1, 3));
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

        room_commander.check_and_swap_stage2(0, None).await.unwrap();

        for player_rx in players_rxs.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(
                received_event,
                RoomEvent::PowerUsed(Power::CheckAndSwapStage2(_, _), 0, None, Some(1), Some(3))
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
                !matches!(received_event, RoomEvent::CardDiscarded(1, card) if card == cards_1[2])
            );
        }
    }

    #[tokio::test]
    async fn throw_own_same_card() {
        let (mut server, _, mut players_rxs) = get_basic_server();
        server.state = State::StartTurn(0);
        server.deck.discard(Card::Clubs(1));
        server
            .players
            .get_mut(&1)
            .unwrap()
            .cards
            .push(Card::Diamonds(1));

        server.throw_same_card(1, 1, 0).unwrap();

        for player_rx in players_rxs.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(
                received_event,
                RoomEvent::SameCardAttempt(
                    1,
                    1,
                    0,
                    Some(Card::Diamonds(1)),
                    SameCardResult::Success
                )
            ));
        }

        assert!(server.players.get(&1).unwrap().cards.is_empty());
    }

    #[tokio::test]
    async fn throw_someone_else_same_card() {
        let (mut server, _, mut players_rxs) = get_basic_server();
        server.state = State::StartTurn(0);
        server.deck.discard(Card::Clubs(1));
        server
            .players
            .get_mut(&0)
            .unwrap()
            .cards
            .push(Card::Diamonds(1));
        server
            .players
            .get_mut(&1)
            .unwrap()
            .cards
            .extend_from_slice(&[Card::Hearts(2), Card::Hearts(3)]);

        server.throw_same_card(1, 0, 0).unwrap();

        for player_rx in players_rxs.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(
                received_event,
                RoomEvent::SameCardAttempt(
                    1,
                    0,
                    0,
                    Some(Card::Diamonds(1)),
                    SameCardResult::Success
                )
            ));
        }

        assert!(server.players.get(&0).unwrap().cards.is_empty());
        assert!(matches!(
            server.state.clone(),
            State::PauseForSameCardThrow(1, 0, 0, state) if *state == State::StartTurn(0)));

        server.select_card_to_give_away(1, 0).unwrap();

        for player_rx in players_rxs.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(
                received_event,
                RoomEvent::CardReplaced(1, 0, 0, 0)
            ));
        }

        assert!(server.players.get(&0).unwrap().cards.len() == 1);
        assert!(server.players.get(&0).unwrap().cards[0] == Card::Hearts(2));
        assert!(server.players.get(&1).unwrap().cards.len() == 1);
        assert!(server.players.get(&1).unwrap().cards[0] == Card::Hearts(3));

        assert!(matches!(server.state, State::StartTurn(0)));
    }

    #[tokio::test]
    async fn throw_same_card_penalty_when_is_not_the_same() {
        let state = State::StartTurn(5);
        let mut deck = Deck::new();
        deck.discard(Card::Hearts(1));
        let cards_1 = vec![
            Card::Clubs(5),
            Card::Clubs(2),
            Card::Clubs(3),
            Card::Clubs(4),
        ];

        let (room_commander, mut players_rxs) =
            init_specific_game_room(5, state, deck, cards_1.clone(), None);

        room_commander.throw_same_card(5, 0, 0).await.unwrap();

        for player_rx in players_rxs.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(
                received_event,
                RoomEvent::SameCardAttempt(5,0,0,Some(card),SameCardResult::NotTheSame) if card==cards_1[0]
            ));
        }

        room_commander.draw_card(5).await.unwrap();
        players_rxs
            .iter_mut()
            .for_each(|rx| while rx.try_recv().is_ok() {});
        room_commander.swap_card(5, 0).await.unwrap();
        for player_rx in players_rxs.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::CardSwapped(5, 0)));
            let received_event = get_nth_event(player_rx, 2).await;
            assert!(!matches!(received_event, RoomEvent::CardDiscarded(5, _)));
        }
    }

    #[tokio::test]
    async fn throw_same_card_penalty_when_someone_already_threw_one() {
        let (mut server, _, mut players_rxs) = get_basic_server();
        server.state = State::StartTurn(0);
        server.deck.discard(Card::Clubs(1));
        server
            .players
            .get_mut(&1)
            .unwrap()
            .cards
            .push(Card::Diamonds(1));

        server.throw_same_card(1, 1, 0).unwrap();
        players_rxs
            .iter_mut()
            .for_each(|rx| while rx.try_recv().is_ok() {});
        server.throw_same_card(0, 1, 0).unwrap();

        for player_rx in players_rxs.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(
                received_event,
                RoomEvent::SameCardAttempt(0, 1, 0, None, SameCardResult::TooLate)
            ));
        }

        assert!(server.players.get(&1).unwrap().cards.is_empty());
        assert!(server.players.get(&0).unwrap().cards.len() == 1);
    }

    #[tokio::test]
    async fn go_crabul() {
        let (mut server, _, mut players_rxs) = get_basic_server();
        server.state = State::StartTurn(0);
        server
            .players
            .get_mut(&1)
            .unwrap()
            .cards
            .extend_from_slice(&[Card::Hearts(13), Card::Diamonds(1), Card::Joker]);

        server.go_crabul(0).unwrap();

        for player_rx in players_rxs.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::PlayerWentCrabul(0)));
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::PlayerTurn(1)));
        }

        assert!(matches!(server.crabul_player, Some(0)));
    }

    #[tokio::test]
    async fn cant_crabul_if_someone_else_already_crabul() {
        let (mut server, _, mut players_rxs) = get_basic_server();
        server.state = State::StartTurn(0);
        server
            .players
            .get_mut(&1)
            .unwrap()
            .cards
            .extend_from_slice(&[Card::Hearts(13), Card::Diamonds(1), Card::Joker]);

        server.go_crabul(0).unwrap();
        players_rxs
            .iter_mut()
            .for_each(|rx| while rx.try_recv().is_ok() {});
        assert!(matches!(
            server.go_crabul(1),
            Err(GameError::OperationNotAllowedAtCurrentState)
        ));
    }

    #[tokio::test]
    async fn cant_use_power_on_crabul_player() {
        let (mut server, _, _) = get_basic_server();
        server.crabul_player = Some(1);

        server.state = State::PowerStage(0, Power::PeekOtherCard);
        assert!(matches!(
            server.peek_other_card(0, 1, 0),
            Err(GameError::OperationNotAllowedAtCurrentState)
        ));

        server.state = State::PowerStage(0, Power::BlindSwap);
        assert!(matches!(
            server.blind_swap(0, 0, 1, 0),
            Err(GameError::OperationNotAllowedAtCurrentState)
        ));

        server.state = State::PowerStage(0, Power::CheckAndSwapStage1);
        assert!(matches!(
            server.check_and_swap_stage1(0, 1, 0),
            Err(GameError::OperationNotAllowedAtCurrentState)
        ));
    }

    #[tokio::test]
    async fn end_game_when_turn_reaches_crabul_player() {
        pause();
        let (mut server, room_commander, mut players_rxs) = get_basic_server();

        for (_, player) in server.players.iter_mut() {
            player
                .cards
                .extend_from_slice(&[Card::Clubs(10), Card::Clubs(10)]);
        }

        server.players.get_mut(&0).unwrap().cards.clear();
        server
            .players
            .get_mut(&0)
            .unwrap()
            .cards
            .extend_from_slice(&[Card::Hearts(13), Card::Diamonds(1), Card::Joker]);

        server.crabul_player = Some(0);
        server.current_player_idx = 5;
        server.state = State::StartTurn(5);
        server.draw_card(5).unwrap();
        server.state = State::MiddleTurn(5, Card::Clubs(10));
        players_rxs
            .iter_mut()
            .for_each(|rx| while rx.try_recv().is_ok() {});

        spawn(server.run());

        room_commander.swap_card(5, 0).await.unwrap();

        sleep(FINALIZE_GAME_COUNTDOWN.add(Duration::from_secs(1))).await;

        for player_rx in players_rxs.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::CardSwapped(..)));
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::CardDiscarded(..)));
            let received_event = get_nth_event(player_rx, 1).await;
            if let RoomEvent::GameTerminated(score) = received_event {
                assert!(score.winner == 0);
                assert!(score.scores[0].total_score == 0);
                assert!(score.scores[1].total_score == 20);

                assert!(
                    score.scores[0].cards == vec![Card::Hearts(13), Card::Diamonds(1), Card::Joker]
                );
            } else {
                panic!("Game not terminated");
            }
        }
    }

    #[tokio::test]
    async fn room_terminate_when_game_is_over() {
        pause();
        let (mut server, commander, mut players_rxs) = get_basic_server();

        for (_, player) in server.players.iter_mut() {
            player
                .cards
                .extend_from_slice(&[Card::Clubs(10), Card::Clubs(10)]);
        }

        server.players.get_mut(&0).unwrap().cards.clear();
        server
            .players
            .get_mut(&0)
            .unwrap()
            .cards
            .extend_from_slice(&[Card::Hearts(13), Card::Diamonds(1), Card::Joker]);

        server.crabul_player = Some(0);
        server.current_player_idx = 5;
        server.state = State::StartTurn(5);
        server.draw_card(5).unwrap();
        server.state = State::MiddleTurn(5, Card::Clubs(10));
        players_rxs
            .iter_mut()
            .for_each(|rx| while rx.try_recv().is_ok() {});

        spawn(server.run());
        commander.swap_card(5, 0).await.unwrap();

        sleep(FINALIZE_GAME_COUNTDOWN.add(Duration::from_secs(1))).await;

        for player_rx in players_rxs.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::CardSwapped(..)));
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::CardDiscarded(..)));
            let received_event = get_nth_event(player_rx, 1).await;
            if let RoomEvent::GameTerminated(score) = received_event {
                assert!(score.winner == 0);
                assert!(score.scores[0].total_score == 0);
                assert!(score.scores[1].total_score == 20);

                assert!(
                    score.scores[0].cards == vec![Card::Hearts(13), Card::Diamonds(1), Card::Joker]
                );
            } else {
                panic!("Game not terminated");
            }
            assert!(player_rx.is_closed());
        }
    }

    #[tokio::test]
    #[should_panic]
    async fn room_terminate_when_no_players_left() {
        let (server, commander, _) = get_basic_server();
        spawn(server.run());
        for i in 0..6 {
            commander.remove_player(i).await;
        }
        let _ = commander.new_player("test".into()).await;
    }

    #[tokio::test]
    async fn turn_timeout() {
        pause();
        let (mut server, commander, mut players_rxs) = get_basic_server();
        for (_, player) in server.players.iter_mut() {
            player
                .cards
                .extend_from_slice(&[Card::Clubs(10), Card::Clubs(10)]);
        }
        let special_deck = deck::testing_deck(vec![Card::Clubs(2), Card::Clubs(3)]);
        server.deck = special_deck;
        server.current_player_idx = 0;
        server.state = State::StartTurn(0);
        server.draw_card(0).unwrap();
        spawn(server.run());
        commander.swap_card(0, 0).await.unwrap();

        players_rxs
            .iter_mut()
            .for_each(|rx| while rx.try_recv().is_ok() {});

        sleep(TURN_COUNTDOWN.add(Duration::from_secs(1))).await;

        for player_rx in players_rxs.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::TurnEndedByTimeout(1)));
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::CardWasDrawn(1)));
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(
                received_event,
                RoomEvent::CardDiscarded(1, Card::Clubs(2))
            ));
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::PlayerTurn(2)));
        }
    }

    #[tokio::test]
    async fn turn_timeout_when_card_has_power() {
        pause();
        let (mut server, commander, mut players_rxs) = get_basic_server();
        for (_, player) in server.players.iter_mut() {
            player
                .cards
                .extend_from_slice(&[Card::Clubs(10), Card::Clubs(10)]);
        }
        let special_deck = deck::testing_deck(vec![Card::Clubs(7), Card::Clubs(3)]);
        server.deck = special_deck;
        server.current_player_idx = 0;
        server.state = State::StartTurn(0);
        server.draw_card(0).unwrap();
        spawn(server.run());
        commander.swap_card(0, 0).await.unwrap();

        players_rxs
            .iter_mut()
            .for_each(|rx| while rx.try_recv().is_ok() {});

        sleep(TURN_COUNTDOWN.add(Duration::from_secs(1))).await;

        for player_rx in players_rxs.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::TurnEndedByTimeout(1)));
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::CardWasDrawn(1)));
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(
                received_event,
                RoomEvent::CardDiscarded(1, Card::Clubs(7))
            ));
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(
                received_event,
                RoomEvent::PowerDiscarded(1, Power::PeekOwnCard)
            ));
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::PlayerTurn(2)));
        }
    }

    #[tokio::test]
    async fn turn_timeout_when_card_has_power_and_power_is_blind_swap() {
        pause();
        let (mut server, commander, mut players_rxs) = get_basic_server();
        for (_, player) in server.players.iter_mut() {
            player
                .cards
                .extend_from_slice(&[Card::Clubs(10), Card::Clubs(10)]);
        }
        let special_deck = deck::testing_deck(vec![Card::Clubs(11), Card::Clubs(3)]);
        server.deck = special_deck;
        server.current_player_idx = 0;
        server.state = State::StartTurn(0);
        server.draw_card(0).unwrap();
        spawn(server.run());
        commander.swap_card(0, 0).await.unwrap();

        players_rxs
            .iter_mut()
            .for_each(|rx| while rx.try_recv().is_ok() {});

        sleep(TURN_COUNTDOWN.add(Duration::from_secs(1))).await;

        for player_rx in players_rxs.iter_mut() {
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::TurnEndedByTimeout(1)));
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::CardWasDrawn(1)));
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(
                received_event,
                RoomEvent::CardDiscarded(1, Card::Clubs(11))
            ));
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::ForcedBlindSwap(1, ..)));
            let received_event = get_nth_event(player_rx, 1).await;
            assert!(matches!(received_event, RoomEvent::PlayerTurn(2)));
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
            same_card_thrown: false,
            current_player_idx,
            crabul_player: None,
            current_count_down: None,
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

    fn get_basic_server() -> (RoomServer, RoomCommander, Vec<UnboundedReceiver<RoomEvent>>) {
        let (mut server, commander) = RoomServer::new();
        let mut player_rxs = vec![];

        for i in 0..6 {
            let (tx, rx) = mpsc::unbounded_channel();
            server.players.insert(
                i,
                Player {
                    name: format!("p{i}"),
                    tx,
                    cards: vec![],
                    ready: true,
                },
            );
            server.turn_order.insert(i as usize, i);
            player_rxs.push(rx);
        }

        (server, commander, player_rxs)
    }
}
