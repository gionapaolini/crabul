import { WebSocketDataType } from "@/models/WebSocketDataType";
import { create } from "zustand";
import { useGameStore } from "./gameStore";
import { useRoomStore } from "./roomStore";

type WebSocketMessage<T = any> = {
    [K in WebSocketDataType]: T;
};

type MessageHandlers = {
    [K in WebSocketDataType]?: (data: any) => void;
};

interface SocketState {
    socket: any;
    message: string;
    isConnected: boolean;
    navigation: string;
}
interface SocketActions {
    handleWebSocketMessage: (data: any) => void;
    connect: (endpoint: string) => void;
    setNavigation: (path: string) => void;
}

export const useSocketStore = create<SocketState & SocketActions>((set, get) => ({
    socket: null,
    message: "",
    isConnected: false,
    navigation: "",
    connect: (endpoint: string) => {
        const { location } = window;
        const proto = location.protocol.startsWith("https") ? "wss" : "ws";
        const host = "49.13.158.245:5000"; // location.host;
        const wsUri = `${proto}://${host}/${endpoint}?name=${useRoomStore.getState().myPlayerName}`;

        const ws = new WebSocket(wsUri);

        ws.onopen = () => {
            set({ isConnected: true });
        };

        ws.onmessage = (ev) => {
            const msg = JSON.parse(ev.data);
            set({ message: msg });
        };

        ws.onclose = () => {
            set({ isConnected: false, socket: null });
        };

        set({ socket: ws });
    },
    handleWebSocketMessage: (message: WebSocketMessage) => {
        const messageHandlers: MessageHandlers = {
            // Waiting Room Actions
            [WebSocketDataType.PlayerJoined]: useRoomStore.getState().handlePlayerJoined,
            [WebSocketDataType.PlayerLeft]: useRoomStore.getState().handlePlayerLeft,
            // Game Actions
            [WebSocketDataType.PeekingPhaseStarted]: useGameStore.getState().handlePickingPhase,
            [WebSocketDataType.PlayerTurn]: useGameStore.getState().handlePlayerTurn,
            [WebSocketDataType.CardWasDrawn]: useGameStore.getState().handleCardWasDrawn,
            [WebSocketDataType.PlayerWentCrabul]: useGameStore.getState().handlePlayerWentCrabul,
            [WebSocketDataType.DrawnCard]: useGameStore.getState().handleDrawnCard,
            [WebSocketDataType.CardSwapped]: useGameStore.getState().handleCardSwapped,
            [WebSocketDataType.CardDiscarded]: useGameStore.getState().handleCardDiscarded,
            [WebSocketDataType.PowerUsed]: useGameStore.getState().handlePowerUsed,
            [WebSocketDataType.PeekedCard]: useGameStore.getState().handlePeekedCard,
            [WebSocketDataType.GameTerminated]: useGameStore.getState().handleGameTerminated,
            [WebSocketDataType.SameCardAttempt]: useGameStore.getState().handleSameCardAttempt,
            [WebSocketDataType.CardReplaced]: useGameStore.getState().handleCardReplaced,
        };

        const messageType = Object.keys(message)[0] as WebSocketDataType;
        const handler = messageHandlers[messageType];

        if (handler) {
            handler(message[messageType]);
        } else {
            console.warn(`No handler found for message type: ${messageType}`);
        }
    },
    setNavigation: (navigation: string) => set({ navigation }),
}));