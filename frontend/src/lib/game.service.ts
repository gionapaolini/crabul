import { useGameStore } from "@/store/gameStore";
import { useRoomStore } from "@/store/roomStore";
import { useSocketStore } from "@/store/socketStore";

export const startGame = ({ cards, players }: {
    cards: any[],
    players: Record<string, string>
}) => {
    useSocketStore.getState().setNavigation("/play")
    useGameStore.getState().createNotification('Game has started');

    const playerCardContainer = document.getElementById("player-card-container") as HTMLElement;
    const mainPlayerCardContainer = document.getElementById("main-player-card-container") as HTMLElement;

    const myPlayerId = useRoomStore.getState().myPlayerId;
    const playersWithoutMe = Object.fromEntries(
        Object.entries(players).filter(([playerId]) => playerId !== myPlayerId?.toString())
    );

    for (let key in playersWithoutMe) {
        const newPlayerCards = `
        <div id="container-player-${key}" class="bg-secondary text-black p-3 rounded position-relative">
            <img id="crabul-badge-${key}" src="crabul-badge.svg" alt="CRABUL!" class="crabul-badge d-none">
            <div class="fw-bold">${players[key]}</div>
            <div id="cards-container-player-${key}"class="flex gap-3 p-2 mt-2">
                <img id="player-${key}-card-0" src="cards/retro.svg" alt="Card" class="img-fluid hover:scale-105" style="width: 60px;" draggable="true">
                <img id="player-${key}-card-1" src="cards/retro.svg" alt="Card" class="img-fluid hover:scale-105" style="width: 60px;" draggable="true">
                <img id="player-${key}-card-2" src="cards/retro.svg" alt="Card" class="img-fluid hover:scale-105" style="width: 60px;" draggable="true">
                <img id="player-${key}-card-3" src="cards/retro.svg" alt="Card" class="img-fluid hover:scale-105" style="width: 60px;" draggable="true">
            </div>
        </div>`;

        playerCardContainer.insertAdjacentHTML("beforeend", newPlayerCards);
    }

    for (let i in [1, 2, 3, 4]) {
        const playerCard = `
            <img id="main-card-${i}" src="cards/retro.svg" alt="Card" class="img-fluid hover:scale-105" style="width: 70px;" draggable="true">    
        `;

        mainPlayerCardContainer.insertAdjacentHTML("beforeend", playerCard);
    }

    const mainCard0 = document.getElementById("main-card-0") as HTMLImageElement;
    const mainCard1 = document.getElementById("main-card-1") as HTMLImageElement;
    mainCard0.src = getCardImage(cards[0]);
    mainCard1.src = getCardImage(cards[1]);

    initializeDragAndDrop();
}

const initializeDragAndDrop = () => {
    const socket = useSocketStore.getState().socket;
    const myPlayerId = useRoomStore.getState().myPlayerId

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

            socket?.send(`/throw ${other_player_id} ${other_card_idx}`);
        } else {
            const card_idx = draggedCardId?.split("-")[2];
            socket?.send(`/throw ${myPlayerId} ${card_idx}`);
        }
    });
}

const initializeDraggableCard = (card: any) => {
    const socket = useSocketStore.getState().socket;
    const powerContainer = document.getElementById("power-container");

    card.addEventListener('dragstart', (e: any) => {
        e.dataTransfer.setData('text/plain', card.id); // Store dragged card's ID
    });

    card.addEventListener('click', (e: any) => {
        const powerState = useGameStore.getState().powerState

        if (powerState == "PeekOwnCard") {
            const card_idx = card.id.split("-")[2];
            socket?.send(`/pow1 ${card_idx}`);
        }
        if (powerState == "CheckAndSwapStage2") {
            const card_idx = card.id.split("-")[2];
            socket?.send(`/pow4_2 ${card_idx}`);
        }
        if (powerState == "Swap") {
            const card_idx = card.id.split("-")[2];
            socket?.send(`/swap ${card_idx}`);
            powerContainer && (powerContainer.innerText = "");
        }
        if (powerState == "ChoosingCardToGive") {
            const card_idx = card.id.split("-")[2];
            socket?.send(`/throw_2 ${card_idx}`);
        }
    });
}

const initializeDroppableCard = (card: any) => {
    const socket = useSocketStore.getState().socket;

    card.addEventListener('click', (e: any) => {
        const powerState = useGameStore.getState().powerState

        if (powerState == "PeekOtherCard") {
            const split = card.id.split("-");
            const other_player_id = split[1];
            const other_card_idx = split[3];
            socket?.send(`/pow2 ${other_player_id} ${other_card_idx}`);
        }
        if (powerState == "CheckAndSwapStage1") {
            const split = card.id.split("-");
            const other_player_id = split[1];
            const other_card_idx = split[3];
            socket?.send(`/pow4_1 ${other_player_id} ${other_card_idx}`);
        }
    });

    card.addEventListener('dragstart', (e: any) => {
        e.dataTransfer.setData('text/plain', card.id); // Store dragged card's ID
    });

    card.addEventListener('dragover', (e: any) => {
        e.preventDefault(); // Allow drop
    });

    card.addEventListener('drop', (e: any) => {
        const powerState = useGameStore.getState().powerState

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
            socket?.send(`/pow3 ${card_idx} ${other_player_id} ${other_card_idx}`);
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
            <img id="player-${userId}-card-${cards.length}" src="cards/retro.svg" alt="Card" class="img-fluid hover:scale-105" style="width: 60px;" draggable="true">
        `;

        const container = document.getElementById(`cards-container-player-${userId}`) as HTMLElement;
        container.insertAdjacentHTML("beforeend", playerCard);

        initializeDroppableCard(document.getElementById(`player-${userId}-card-${cards.length}`));
    } else {
        let cards = document.querySelectorAll(`[id^="main-card-"]`);

        const playerCard = `
            <img id="main-card-${cards.length}" src="cards/retro.svg" alt="Card" class="img-fluid hover:scale-105" style="width: 70px;" draggable="true">    
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
    const createNotification = useGameStore.getState().createNotification;
    const players = useRoomStore.getState().players_list

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

export const showGameResults = ({ data, myPlayerId, myPlayerName, players }: any) => {
    const gameResultModal = document.getElementById("game-result-modal") as HTMLElement;
    const scoresContainer = document.getElementById("game-scores") as HTMLElement;

    // Clear previous content
    scoresContainer.innerHTML = "";

    // Parse the data
    // const gameData = data.GameTerminated;
    const gameData = data;
    const winner = gameData.winner == myPlayerId ? myPlayerName : players[gameData.winner];

    // Create a heading for the winner
    const winnerHeading = document.createElement("h2");
    winnerHeading.textContent = `Winner: Player ${winner}`;
    scoresContainer.appendChild(winnerHeading);

    // Create a table for scores
    const scoresTable = document.createElement("table");
    scoresTable.style.width = "100%";
    scoresTable.style.borderCollapse = "collapse";

    // Add table headers
    const headers = document.createElement("tr");
    headers.innerHTML = `
        <th style="border: 1px solid #ddd; padding: 8px;">Player</th>
        <th style="border: 1px solid #ddd; padding: 8px;">Cards</th>
        <th style="border: 1px solid #ddd; padding: 8px;">Total Score</th>
    `;
    scoresTable.appendChild(headers);

    const suitMap: any = {
        Clubs: 0,
        Diamonds: 1,
        Hearts: 2,
        Spade: 3
    };

    // Add rows for each player
    gameData.scores.forEach((score: any) => {
        const row = document.createElement("tr");

        // Player ID
        const playerIdCell = document.createElement("td");
        playerIdCell.style.border = "1px solid #ddd";
        playerIdCell.style.padding = "8px";
        playerIdCell.textContent = score.player_id == myPlayerId ? myPlayerName : players[score.player_id];
        row.appendChild(playerIdCell);

        // Cards
        const cardsCell = document.createElement("td");
        cardsCell.style.border = "1px solid #ddd";
        cardsCell.style.padding = "8px";
        score.cards.forEach((card: any) => {
            const suit: any = Object.keys(card)[0] as any;
            const value = card[suit];
            const suitId = suitMap[suit];
            const cardImg = document.createElement("img");
            if (card == "Joker") {
                cardImg.src = `cards/joker.svg`;
            } else {
                cardImg.src = `cards/${suitId}_${value}.svg`;
            }
            cardImg.alt = `${value} of ${suit}`;
            cardImg.style.width = "60px"; // Adjust size as needed
            cardImg.style.marginRight = "5px";

            cardsCell.appendChild(cardImg);
        });
        row.appendChild(cardsCell);

        // Total Score
        const totalScoreCell = document.createElement("td");
        totalScoreCell.style.border = "1px solid #ddd";
        totalScoreCell.style.padding = "8px";
        totalScoreCell.textContent = score.total_score;
        row.appendChild(totalScoreCell);

        scoresTable.appendChild(row);
    });

    scoresContainer.appendChild(scoresTable);

    // Show the modal
    gameResultModal.style.display = "flex";
}
