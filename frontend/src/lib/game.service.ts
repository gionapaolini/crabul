import { useWebSocket } from "@/hooks/useWebSocket";
import { useGameStore } from "@/store/gameStore";
import { useRoomStore } from "@/store/roomStore";
import { Player } from "@/views/Game/WaitingRoom";

export const startGame = ({ cards, players }: {
    cards: any[],
    players: Player[]
}) => {
    useGameStore().createNotification('Game has started');
    // waitingRoom.remove(); Route to game room
    const playerCardContainer = document.getElementById("player-card-container") as HTMLElement;
    const mainPlayerCardContainer = document.getElementById("main-player-card-container") as HTMLElement;;

    for (let key in players) {
        const newPlayerCards = `
        <div id="container-player-${key}" class="bg-secondary text-black p-3 rounded position-relative">
            <img id="crabul-badge-${key}" src="crabul-badge.svg" alt="CRABUL!" class="crabul-badge d-none">
            <div class="fw-bold">${players[key]}</div>
            <div id="cards-container-player-${key}"class="d-flex gap-3 p-2 mt-2">
                <img id="player-${key}-card-0" src="cards/retro.svg" alt="Card" class="img-fluid hover-zoom" style="width: 60px;" draggable="true">
                <img id="player-${key}-card-1" src="cards/retro.svg" alt="Card" class="img-fluid hover-zoom" style="width: 60px;" draggable="true">
                <img id="player-${key}-card-2" src="cards/retro.svg" alt="Card" class="img-fluid hover-zoom" style="width: 60px;" draggable="true">
                <img id="player-${key}-card-3" src="cards/retro.svg" alt="Card" class="img-fluid hover-zoom" style="width: 60px;" draggable="true">
            </div>
        </div>`;

        // TODO Add cards to dom
        playerCardContainer?.insertAdjacentHTML("beforeend", newPlayerCards);
    }

    for (let i in [...Array(4).keys()]) {
        const playerCard = `
            <img id="main-card-${i}" src="cards/retro.svg" alt="Card" class="img-fluid hover-zoom" style="width: 70px;" draggable="true">    
        `;

        // TODO Add cards to dom
        mainPlayerCardContainer.insertAdjacentHTML("beforeend", playerCard);
    }

    const mainCard0 = document.getElementById("main-card-0") as HTMLImageElement;
    const mainCard1 = document.getElementById("main-card-1") as HTMLImageElement;
    mainCard0.src = getCardImage(cards[0]);
    mainCard1.src = getCardImage(cards[1]);

    // gameRoom.style.display = "block";

    initializeDragAndDrop();
}

const initializeDragAndDrop = () => {
    const { socket } = useWebSocket();

    const draggableCards = document.querySelectorAll('#main-player-card-container img[draggable="true"]');
    const droppableCards = document.querySelectorAll('[id^="cards-container-player-"] img');
    const droppableDiscardPile = document.getElementById('discard-pile');

    draggableCards.forEach(card => {
        initializeDraggableCard(card);
    });

    droppableCards.forEach(card => {
        initializeDroppableCard(card);
    });

    droppableDiscardPile?.addEventListener('dragover', (e) => {
        e.preventDefault(); // Allow drop
    });

    droppableDiscardPile?.addEventListener('drop', (e) => {
        e.preventDefault();

        const draggedCardId = e.dataTransfer?.getData('text/plain'); // Get dragged card ID

        if (draggedCardId?.startsWith("player-")) {
            const split = draggedCardId?.split("-");
            const other_player_id = split[1];
            const other_card_idx = split[3];

            socket.send(`/throw ${other_player_id} ${other_card_idx}`);
        } else {
            const card_idx = draggedCardId?.split("-")[2];
            socket.send(`/throw ${useRoomStore().myPlayerId} ${card_idx}`);
        }
    });
}

const initializeDraggableCard = (card: any) => {
    const { socket } = useWebSocket();
    const powerState = useGameStore().powerState;
    const powerContainer = document.getElementById("power-container");

    card.addEventListener('dragstart', (e: any) => {
        e.dataTransfer.setData('text/plain', card.id); // Store dragged card's ID
    });

    card.addEventListener('click', (e: any) => {
        if (powerState == "PeekOwnCard") {
            const card_idx = card.id.split("-")[2];
            socket.send(`/pow1 ${card_idx}`);
        }
        if (powerState == "CheckAndSwapStage2") {
            const card_idx = card.id.split("-")[2];
            socket.send(`/pow4_2 ${card_idx}`);
        }
        if (powerState == "Swap") {
            const card_idx = card.id.split("-")[2];
            socket.send(`/swap ${card_idx}`);
            powerContainer && (powerContainer.innerText = "");
        }
        if (powerState == "ChoosingCardToGive") {
            const card_idx = card.id.split("-")[2];
            socket.send(`/throw_2 ${card_idx}`);
        }
    });
}

const initializeDroppableCard = (card: any) => {
    const { socket } = useWebSocket();
    const powerState = useGameStore().powerState;

    card.addEventListener('click', (e: any) => {
        if (powerState == "PeekOtherCard") {
            const split = card.id.split("-");
            const other_player_id = split[1];
            const other_card_idx = split[3];
            socket.send(`/pow2 ${other_player_id} ${other_card_idx}`);
        }
        if (powerState == "CheckAndSwapStage1") {
            const split = card.id.split("-");
            const other_player_id = split[1];
            const other_card_idx = split[3];
            socket.send(`/pow4_1 ${other_player_id} ${other_card_idx}`);
        }
    });

    card.addEventListener('dragstart', (e: any) => {
        e.dataTransfer.setData('text/plain', card.id); // Store dragged card's ID
    });

    card.addEventListener('dragover', (e: any) => {
        e.preventDefault(); // Allow drop
    });

    card.addEventListener('drop', (e: any) => {
        e.preventDefault();

        const draggedCardId = e.dataTransfer.getData('text/plain'); // Get dragged card ID
        const dropTargetId = card.id; // Get target card ID

        if (draggedCardId.startsWith("player-")) {
            return;
        }

        const card_idx = draggedCardId.split("-")[2];
        const split = dropTargetId.split("-");
        const other_player_id = split[1];
        const other_card_idx = split[3];

        if (powerState == "BlindSwap") { // blind_swap
            socket.send(`/pow3 ${card_idx} ${other_player_id} ${other_card_idx}`);
        }
    });
}

export const getCardImage = (card: any): string => {
    if (card === "Joker") {
        return "cards/joker.svg";
    }
    if ("Clubs" in card) {
        return `cards/0_${card["Clubs"]}.svg`;
    }
    if ("Diamonds" in card) {
        return `cards/1_${card["Diamonds"]}.svg`;
    }
    if ("Hearts" in card) {
        return `cards/2_${card["Hearts"]}.svg`;
    }
    if ("Spade" in card) {
        return `cards/3_${card["Spade"]}.svg`;
    }
    alert(`Card not recognized: ${JSON.stringify(card)}`)
    return ""
}

export const coverCards = () => {
    const cards = document.querySelectorAll('[id^="main-card-"]') as NodeListOf<HTMLImageElement>;
    // Change the src of each matched element (assuming they're images)
    cards.forEach(card => {
        card.src = "cards/retro.svg";
    });
}

export const highlightCurrentPlayer = ({ id, myPlayerId }: { id: string; myPlayerId: number }) => {
    const mainPlayerCardContainer = document.getElementById("main-player-card-container") as HTMLElement;

    let playerDiv: HTMLElement;
    // console.log("Current player turn is " + id + " player id is: " + userPlayerId);
    mainPlayerCardContainer.classList.remove("highlight-shadow");
    const playerDivs = document.querySelectorAll('[id^="container-player-"]');

    playerDivs.forEach(pdiv => {
        pdiv.classList.remove("highlight-shadow");
    });

    if (+id === +myPlayerId) {
        playerDiv = mainPlayerCardContainer;
    } else {
        playerDiv = document.getElementById(`container-player-${id}`) as HTMLElement;

    }
    playerDiv.classList.add("highlight-shadow");
}


export const addCard = ({ userId, players }: { userId: any, players: any }) => {
    if (userId in players) {
        let cards = document.querySelectorAll(`[id^="player-${userId}-card-"]`);
        const playerCard = `
            <img id="player-${userId}-card-${cards.length}" src="cards/retro.svg" alt="Card" class="img-fluid hover-zoom" style="width: 60px;" draggable="true">
        `;

        const container = document.getElementById(`cards-container-player-${userId}`) as HTMLElement;
        container.insertAdjacentHTML("beforeend", playerCard);

        initializeDroppableCard(document.getElementById(`player-${userId}-card-${cards.length}`));
    } else {
        let cards = document.querySelectorAll(`[id^="main-card-"]`);

        const playerCard = `
            <img id="main-card-${cards.length}" src="cards/retro.svg" alt="Card" class="img-fluid hover-zoom" style="width: 70px;" draggable="true">    
        `;
        const mainPlayerCardContainer = document.getElementById("main-player-card-container") as HTMLElement;
        mainPlayerCardContainer.insertAdjacentHTML("beforeend", playerCard);
        initializeDraggableCard(document.getElementById(`main-card-${cards.length}`));
    }
}

export const removeCard = ({ userId, players }: { userId: any, players: any }) => {
    if (userId in players) {
        let cards = document.querySelectorAll(`[id^="player-${userId}-card-"]`);
        const card = document.getElementById(`player-${userId}-card-${cards.length - 1}`) as HTMLElement;
        card.remove();
    } else {
        let cards = document.querySelectorAll(`[id^="main-card-"]`);
        const card = document.getElementById(`main-card-${cards.length - 1}`) as HTMLElement;
        card.remove();
    }
}

export const sendPowerUsedNotification = ({ power }: { power: any }) => {
    const players = useRoomStore().players_list;
    const createNotification = useGameStore().createNotification;

    if (power[0] === "PeekOwnCard") {
        if (power[1] in players) {
            createNotification(`Player ${players[power[1]]} peeked his own card at position ${power[2] + 1}`);
        }
        return;
    }
    if (power[0] === "PeekOtherCard") {
        if (power[3] in players) {
            createNotification(
                `Player ${players[power[1]]} peeked card ${power[4] + 1} of player ${players[power[3]]}`);
        } else {
            createNotification(`Player ${players[power[1]]} peeked your card ${power[4] + 1}`);
        }
        return;
    }
    if (power[0] === "BlindSwap") {
        if (power[3] in players) {
            createNotification(
                `Player ${players[power[1]]} blind swapped card ${power[2] + 1} with card ${power[4] + 1} of player ${players[power[3]]}`
            );
        } else {
            createNotification(
                `Player ${players[power[1]]} blind swapped card ${power[2] + 1} with your card ${power[4] + 1}`
            );
        }
        return;
    }

    if (power[0] === "CheckAndSwapStage1") {
        if (power[3] in players) {
            createNotification(
                `Player ${players[power[1]]} peeked card ${power[4] + 1} of player ${players[power[3]]} and is deciding whether to swap`
            );
        } else {
            createNotification(
                `Player ${players[power[1]]} peeked your card ${power[2] + 1} and is deciding whether to swap`
            );
        }
        return;
    }

    if ("CheckAndSwapStage2" in power[0]) {
        if (power[2] != null) {
            if (power[3] in players) {
                createNotification(
                    `Player ${players[power[1]]} swapped card ${power[2] + 1} with card ${power[4] + 1} of player ${players[power[3]]}`
                );
            } else {
                createNotification(
                    `Player ${players[power[1]]} swapped card ${power[2] + 1} with your card ${power[4] + 1}`
                );
            }
        } else {
            createNotification(`Player ${players[power[1]]} decided not to swap`);
        }
        return;
    }
}
