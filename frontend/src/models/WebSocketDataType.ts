export enum WebSocketDataType {
  OperationNotAllowedAtCurrentState = "OperationNotAllowedAtCurrentState",
  PlayerJoined = "PlayerJoined",
  PlayerLeft = "PlayerLeft",
  PeekingPhaseStarted = "PeekingPhaseStarted",
  PlayerTurn = "PlayerTurn",
  CardWasDrawn = "CardWasDrawn",
  PlayerWentCrabul = "PlayerWentCrabul",
  DrawnCard = "DrawnCard",
  CardSwapped = "CardSwapped",
  CardDiscarded = "CardDiscarded",
  PowerActivated = "PowerActivated",
  PowerUsed = "PowerUsed",
  PeekedCard = "PeekedCard",
  GameTerminated = "GameTerminated",
  SameCardAttempt = "SameCardAttempt",
  CardReplaced = "CardReplaced",
}

// Types
export interface PlayerJoinedRes {
  player_id: number;
  player_name: string;
  room_id: number;
  player_list: Record<string, string>;
}