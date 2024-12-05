import { createGameHandlers } from "@/lib/game-messages.handler";
import { WebSocketDataType } from "@/models/WebSocketDataType";
import { useCallback, useMemo, useState } from "react";

type WebSocketMessage<T = any> = {
  [K in WebSocketDataType]: T;
};

type MessageHandlers = {
  [K in WebSocketDataType]?: (data: any) => void;
};

export const useGameState = () => {
  const [state, setState] = useState<{
    powers: any[];
  }>({
    powers: [],
  });
  
  const handlers = useMemo(() => createGameHandlers(setState), [setState]);

  const messageHandlers: MessageHandlers = {
    [WebSocketDataType.PeekingPhaseStarted]: handlers.handlePickingPhase,
    [WebSocketDataType.PlayerTurn]: handlers.handlePlayerTurn,
    [WebSocketDataType.CardWasDrawn]: handlers.handleCardWasDrawn,
    [WebSocketDataType.PlayerWentCrabul]: handlers.handlePlayerWentCrabul,
    [WebSocketDataType.DrawnCard]: handlers.handleDrawnCard,
    [WebSocketDataType.CardSwapped]: handlers.handleCardSwapped,
    [WebSocketDataType.CardDiscarded]: handlers.handleCardDiscarded,
    [WebSocketDataType.PowerUsed]: handlers.handlePowerUsed,
    [WebSocketDataType.PeekedCard]: handlers.handlePeekedCard,
    [WebSocketDataType.GameTerminated]: handlers.handleGameTerminated,
    [WebSocketDataType.SameCardAttempt]: handlers.handleSameCardAttempt,
    [WebSocketDataType.CardReplaced]: handlers.handleCardReplaced,
  };

  const handleWebSocketMessage = useCallback((message: WebSocketMessage) => {
    const messageType = Object.keys(message)[0] as WebSocketDataType;
    const handler = messageHandlers[messageType];

    if (handler) {
      handler(message);
    } else {
      console.warn(`No handler found for message type: ${messageType}`);
    }
  }, []);

  return {
    state,
    handleWebSocketMessage,
  };
};
