use std::{num::ParseIntError, pin::pin};

use actix_ws::{AggregatedMessage, AggregatedMessageStream, Session};
use futures_util::future::{select, Either};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    consts::{GameError, PlayerId},
    room::RoomEvent,
    room_commander::RoomCommander,
};

pub struct WsClient {
    player_id: PlayerId,
    room_commander: RoomCommander,
    player_channel: UnboundedReceiver<RoomEvent>,
    stream: AggregatedMessageStream,
    session: Session,
}

impl WsClient {
    pub fn new(
        player_id: PlayerId,
        room_commander: RoomCommander,
        player_channel: UnboundedReceiver<RoomEvent>,
        stream: AggregatedMessageStream,
        session: Session,
    ) -> Self {
        Self {
            player_id,
            room_commander,
            player_channel,
            stream,
            session,
        }
    }

    pub async fn run(mut self) {
        loop {
            let room_message = pin!(self.player_channel.recv());
            let player_message = pin!(self.stream.recv());
            match select(room_message, player_message).await {
                Either::Left((Some(room_event), _)) => {
                    self.session
                        .text(serde_json::to_string(&room_event).unwrap())
                        .await
                        .unwrap();
                },
                Either::Left((None, _)) => {
                    break;
                },
                Either::Right((Some(Ok(msg)), _)) => match msg {
                    AggregatedMessage::Text(msg) => {
                        let msg: &str = &msg;
                        let to_send = match msg {
                            "/start" => self.room_commander.start_game().await,
                            "/ready" => self.room_commander.set_player_ready(self.player_id).await,
                            "/draw" => self.room_commander.draw_card(self.player_id).await,
                            "/discard" => self.room_commander.discard_card(self.player_id).await,
                            "/crabul" => self.room_commander.go_crabul(self.player_id).await,
                            swap_command if swap_command.starts_with("/swap ") => {
                                Self::swap(
                                    self.player_id,
                                    self.room_commander.clone(),
                                    swap_command,
                                )
                                .await
                            }
                            pow1_command if pow1_command.starts_with("/pow1 ") => {
                                Self::pow1(
                                    self.player_id,
                                    self.room_commander.clone(),
                                    pow1_command,
                                )
                                .await
                            }
                            pow2_command if pow2_command.starts_with("/pow2 ") => {
                                Self::pow2(
                                    self.player_id,
                                    self.room_commander.clone(),
                                    pow2_command,
                                )
                                .await
                            }
                            pow3_command if pow3_command.starts_with("/pow3 ") => {
                                Self::pow3(
                                    self.player_id,
                                    self.room_commander.clone(),
                                    pow3_command,
                                )
                                .await
                            }
                            pow4_1_command if pow4_1_command.starts_with("/pow4_1 ") => {
                                Self::pow4_1(
                                    self.player_id,
                                    self.room_commander.clone(),
                                    pow4_1_command,
                                )
                                .await
                            }
                            pow4_2_command if pow4_2_command.starts_with("/pow4_2 ") => {
                                Self::pow4_2(
                                    self.player_id,
                                    self.room_commander.clone(),
                                    pow4_2_command,
                                )
                                .await
                            }
                            throw_command if throw_command.starts_with("/throw ") => {
                                Self::throw(
                                    self.player_id,
                                    self.room_commander.clone(),
                                    throw_command,
                                )
                                .await
                            }
                            throw_2_command if throw_2_command.starts_with("/throw_2 ") => {
                                Self::throw_2(
                                    self.player_id,
                                    self.room_commander.clone(),
                                    throw_2_command,
                                )
                                .await
                            }
                            _ => {
                                self.session.text("Command not recognized").await.unwrap();
                                Ok(())
                            }
                        };
                        if let Err(err) = to_send {
                            self.session
                                .text(serde_json::to_string(&err).unwrap())
                                .await
                                .unwrap();
                        }
                    }
                    AggregatedMessage::Close(_) => {
                        let _ = self.room_commander.remove_player(self.player_id).await;
                        break;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    async fn swap(
        player_id: PlayerId,
        room_commander: RoomCommander,
        command: &str,
    ) -> Result<(), GameError> {
        if let Ok(params) = Self::parse_command(command) {
            if let Some(card_idx) = params.first() {
                return room_commander.swap_card(player_id, *card_idx).await;
            }
        }
        Err(GameError::UnableToParseCommand)
    }
    async fn pow1(
        player_id: PlayerId,
        room_commander: RoomCommander,
        command: &str,
    ) -> Result<(), GameError> {
        if let Ok(params) = Self::parse_command(command) {
            if let Some(card_idx) = params.first() {
                return room_commander.peek_own_card(player_id, *card_idx).await;
            }
        }
        Err(GameError::UnableToParseCommand)
    }

    async fn pow2(
        player_id: PlayerId,
        room_commander: RoomCommander,
        command: &str,
    ) -> Result<(), GameError> {
        if let Ok(params) = Self::parse_command(command) {
            if let (Some(other_player_id), Some(other_card_idx)) = (params.first(), params.get(1)) {
                return room_commander
                    .peek_other_card(player_id, *other_player_id as PlayerId, *other_card_idx)
                    .await;
            }
        }
        Err(GameError::UnableToParseCommand)
    }

    async fn pow3(
        player_id: PlayerId,
        room_commander: RoomCommander,
        command: &str,
    ) -> Result<(), GameError> {
        if let Ok(params) = Self::parse_command(command) {
            if let (Some(card_idx), Some(other_player_id), Some(other_card_idx)) =
                (params.first(), params.get(1), params.get(2))
            {
                return room_commander
                    .blind_swap(
                        player_id,
                        *card_idx,
                        *other_player_id as PlayerId,
                        *other_card_idx,
                    )
                    .await;
            }
        }
        Err(GameError::UnableToParseCommand)
    }

    async fn pow4_1(
        player_id: PlayerId,
        room_commander: RoomCommander,
        command: &str,
    ) -> Result<(), GameError> {
        if let Ok(params) = Self::parse_command(command) {
            if let (Some(other_player_id), Some(other_card_idx)) = (params.first(), params.get(1)) {
                return room_commander
                    .check_and_swap_stage1(player_id, *other_player_id as PlayerId, *other_card_idx)
                    .await;
            }
        }
        Err(GameError::UnableToParseCommand)
    }

    async fn pow4_2(
        player_id: PlayerId,
        room_commander: RoomCommander,
        command: &str,
    ) -> Result<(), GameError> {
        if let Ok(params) = Self::parse_command(command) {
            if let Some(card_idx) = params.first() {
                return room_commander
                    .check_and_swap_stage2(player_id, Some(*card_idx))
                    .await;
            } else {
                return room_commander.check_and_swap_stage2(player_id, None).await;
            }
        }
        Err(GameError::UnableToParseCommand)
    }
    async fn throw(
        player_id: PlayerId,
        room_commander: RoomCommander,
        command: &str,
    ) -> Result<(), GameError> {
        if let Ok(params) = Self::parse_command(command) {
            if let (Some(other_player_id), Some(other_card_idx)) = (params.first(), params.get(1)) {
                return room_commander
                    .throw_same_card(player_id, *other_player_id as PlayerId, *other_card_idx)
                    .await;
            }
        }
        Err(GameError::UnableToParseCommand)
    }

    async fn throw_2(
        player_id: PlayerId,
        room_commander: RoomCommander,
        command: &str,
    ) -> Result<(), GameError> {
        if let Ok(params) = Self::parse_command(command) {
            if let Some(card_idx) = params.first() {
                return room_commander
                    .select_card_to_give_away(player_id, *card_idx)
                    .await;
            }
        }
        Err(GameError::UnableToParseCommand)
    }

    fn parse_command(command: &str) -> Result<Vec<usize>, ParseIntError> {
        command
            .split(" ")
            .skip(1)
            .filter(|split| !split.is_empty())
            .map(|val| val.parse::<usize>())
            .collect()
    }
}
