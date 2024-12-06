import { create } from "zustand";
import { useRoomStore } from "./roomStore";
import { WebSocketDataType } from "@/models/WebSocketDataType";
import { getCardImage, startGame } from "@/lib/game.service";

type WebSocketMessage<T = any> = {
    [K in WebSocketDataType]: T;
};

type MessageHandlers = {
    [K in WebSocketDataType]?: (data: any) => void;
};

interface GameState {
    // players: Record<string, string>
    powerState: string
    discardPile: string | null
    cards: string[]
    notifications: any
    oldPowerState?: string
    oldPowerContainerText?: string
}

interface GameActions {
    handleWebSocketMessage: (data: any) => void;
    handlePickingPhase: (data: any) => void;
    handlePlayerTurn: (data: any) => void;
    handleCardWasDrawn: (data: any) => void;
    handlePlayerWentCrabul: (data: any) => void;
    handleDrawnCard: (data: any) => void;
    handleCardSwapped: (data: any) => void;
    handleCardDiscarded: (data: any) => void;
    handlePowerActivated: (data: any) => void;
    handlePowerUsed: (data: any) => void;
    handlePeekedCard: (data: any) => void;
    handleGameTerminated: (data: any) => void;
    handleSameCardAttempt: (data: any) => void;
    handleCardReplaced: (data: any) => void;
    createNotification: (data: any) => void;
}

export const useGameStore = create<GameState & GameActions>((set, get) => ({
    // Initial state
    powerState: '',
    discardPile: null,
    cards: [],
    notifications: {},
    handleWebSocketMessage: (message: WebSocketMessage) => {
        const messageHandlers: MessageHandlers = {
            [WebSocketDataType.PeekingPhaseStarted]: get().handlePickingPhase,
            [WebSocketDataType.PlayerTurn]: get().handlePlayerTurn,
            [WebSocketDataType.CardWasDrawn]: get().handleCardWasDrawn,
            [WebSocketDataType.PlayerWentCrabul]: get().handlePlayerWentCrabul,
            [WebSocketDataType.DrawnCard]: get().handleDrawnCard,
            [WebSocketDataType.CardSwapped]: get().handleCardSwapped,
            [WebSocketDataType.CardDiscarded]: get().handleCardDiscarded,
            [WebSocketDataType.PowerUsed]: get().handlePowerUsed,
            [WebSocketDataType.PeekedCard]: get().handlePeekedCard,
            [WebSocketDataType.GameTerminated]: get().handleGameTerminated,
            [WebSocketDataType.SameCardAttempt]: get().handleSameCardAttempt,
            [WebSocketDataType.CardReplaced]: get().handleCardReplaced,
        };

        const messageType = Object.keys(message)[0] as WebSocketDataType;
        const handler = messageHandlers[messageType];

        if (handler) {
            handler(message);
        } else {
            console.warn(`No handler found for message type: ${messageType}`);
        }
    },
    handlePickingPhase: (message: any) => {
        const players = useRoomStore.getState().players;
        console.log("Picking phase started", message);
        startGame({ cards: message, players });
    },
    handlePlayerTurn: (message: any) => {
        const endTurnButton = document.getElementById("end-turn-button") as HTMLButtonElement;
        const drawButton = document.getElementById("draw-card-btn") as HTMLButtonElement;
        const crabulButton = document.getElementById("crabul-btn") as HTMLButtonElement;
        endTurnButton.style.display = "none"; //check a better place to put ths

        const myPlayerId = useRoomStore.getState().myPlayerId;
        const players = useRoomStore.getState().players_list;

        if (message === myPlayerId) {
            get().createNotification(`Your turn`);
            drawButton.disabled = false;
            crabulButton.disabled = false;
        } else {
            get().createNotification(`Player turn: ${players[message]}`);
            drawButton.disabled = true;
            crabulButton.disabled = true;
        }
        coverCards();
        highlightCurrentPlayer(message);
    },
    handleCardWasDrawn: (message: any) => {
        const myPlayerId = useRoomStore.getState().myPlayerId;
        const players = useRoomStore.getState().players_list;
        const drawButton = document.getElementById("draw-card-btn") as HTMLButtonElement;
        const crabulButton = document.getElementById("crabul-btn") as HTMLButtonElement;
        if (message != myPlayerId) {
            get().createNotification(`Player ${players[message]} drew a card from the deck`);
        } else {
            drawButton.disabled = true;
            crabulButton.disabled = true;
        }
    },
    handlePlayerWentCrabul: (message: any) => {
        const myPlayerId = useRoomStore.getState().myPlayerId;
        const players = useRoomStore.getState().players_list;
        const drawButton = document.getElementById("draw-card-btn") as HTMLButtonElement;
        const crabulButton = document.getElementById("crabul-btn") as HTMLButtonElement;

        if (message != myPlayerId) {
            get().createNotification(`Player ${players[message]} went CRABUL!`);
            const badge = document.getElementById(`crabul-badge-${message}`);
            badge?.classList.remove('d-none');
        } else {
            drawButton.disabled = true;
            crabulButton.disabled = true;
        }
    },
    handleDrawnCard: (message: any) => {
        const drawnCard = document.getElementById("drawn-card") as HTMLImageElement;
        const drawnCardModal = document.getElementById("drawn-card-modal") as HTMLElement;

        drawnCard.src = getCardImage(message);
        drawnCardModal.style.display = 'flex';
    },
    handleCardSwapped: (message: any) => {
        const myPlayerId = useRoomStore.getState().myPlayerId;
        const players = useRoomStore.getState().players_list;

        if (message[0] != myPlayerId) {
            get().createNotification(`Player ${players[message[0]]} swapped card ${message[1] + 1}`);
        }
    },
    handleCardDiscarded: (message: any) => {
        const myPlayerId = useRoomStore.getState().myPlayerId;
        const players = useRoomStore.getState().players_list;
        const discardPile = document.getElementById("discard-pile") as HTMLImageElement;

        if (message[0] != myPlayerId) {
            get().createNotification(`Player ${players[message[0]]} discarded a card`);
        }

        discardPile.src = getCardImage(message[1]);
    },
    handlePowerActivated: (message: any) => {
        const myPlayerId = useRoomStore.getState().myPlayerId;
        const players = useRoomStore.getState().players_list;
        const powerContainer = document.getElementById("power-container") as HTMLElement;

        if (message[0] != myPlayerId) {
            get().createNotification(`Player ${players[message[0]]} activated a power: ${message[1]}`);
        } else {
            set({ powerState: message[1] })

            powerContainer.innerText = "POWER " + message[1] + " ACTIVE!";
        }
    },
    handlePowerUsed: (message: any) => {
        const myPlayerId = useRoomStore.getState().myPlayerId;
        const powerContainer = document.getElementById("power-container") as HTMLElement;
        const endTurnButton = document.getElementById("end-turn-button") as HTMLElement;

        if (message[1] != myPlayerId) {
            sendPowerUsedNotification(message);
        } else {
            if (message[0] === "CheckAndSwapStage1") {
                set({ powerState: "CheckAndSwapStage2" })
                powerContainer.innerText = "SWAP WITH ONE OF YOUR CARD OR END TURN";
                endTurnButton.style.display = 'block';
            } else {
                set({ powerState: "" })

                powerContainer.innerText = "";
            }
        }
    },
    handlePeekedCard: (message: any) => {
        const peekedCard = document.getElementById("peeked-card") as HTMLImageElement;
        const peekedCardModal = document.getElementById("peeked-card-modal") as HTMLElement;

        peekedCard.src = getCardImage(message);
        peekedCardModal.style.display = 'flex';
    },
    handleGameTerminated: (message: any) => {
        showGameResults(data);
    },
    handleSameCardAttempt: (message: any) => {
        const players = useRoomStore.getState().players_list;
        const powerContainer = document.getElementById("power-container");
        const discardPile = document.getElementById("discard-pile") as HTMLImageElement;

        if (message[4] !== "Success") {
            addCard(message[0]);
            if (message[0] in players) {
                get().createNotification(`${players[message[0]]} attempted to throw a diplicate unsuccessfully: ${message[4]}, receiving a penalty card`);
            } else {
                get().createNotification(`You attempted to throw a duplicate unsuccessfully: ${message[4]}, receive a penalty card`);
            }
        } else {
            if (message[0] in players) {
                get().createNotification(`Player ${players[message[0]]} threw a duplicate successfully`);
            } else {
                get().createNotification(`You threw a duplicate successfully`);
            }

            removeCard(message[1]);

            if (message[0] != message[1]) {
                set({ powerState: get().oldPowerState })

                const text = powerContainer?.innerText || "";
                set({ oldPowerContainerText: text })

                if (message[0] in players) {
                    set({ powerState: "WaitingForChoosingCardToGive" })

                    powerContainer && (powerContainer.innerText = `PAUSE! Game will resume after ${message[0]} choose the card to substitute`)
                } else {
                    set({ powerState: "ChoosingCardToGive" })
                    powerContainer && (powerContainer.innerText = "PICK ONE OF YOUR CARD TO SUBSTITUTE!")
                }
            }
            discardPile.src = getCardImage(message[3]);
        }
    },
    handleCardReplaced: (message: any) => {
        const players = useRoomStore.getState().players_list;
        const powerContainer = document.getElementById("power-container") as HTMLElement;
        if (message[0] in players) {
            get().createNotification(`${players[message[0]]} have replaced the duplicate with card ${message[1]}`);
        }
        removeCard(message[0]);
        addCard(message[2]);

        set({ powerState: get().oldPowerState })
        powerContainer.innerText = get().oldPowerContainerText || "";
    },
    createNotification: (message: string) => set({ notifications: message })
}))
