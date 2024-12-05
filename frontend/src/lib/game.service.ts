import { useWebSocket } from "@/hooks/useWebSocket";
import { Player } from "@/views/Game/WaitingRoom";

export const startGame = ({ cards, players }: {
    cards: any[],
    players: Player[]
}) => {
    // createNotification('Game has started');
    // waitingRoom.remove(); Route to game room

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
        playerCardContainer.insertAdjacentHTML("beforeend", newPlayerCards);
    }

    for (let i in [...Array(4).keys()]) {
        const playerCard = `
            <img id="main-card-${i}" src="cards/retro.svg" alt="Card" class="img-fluid hover-zoom" style="width: 70px;" draggable="true">    
        `;

        // TODO Add cards to dom
        mainPlayerCardContainer.insertAdjacentHTML("beforeend", playerCard);
    }

    document.getElementById("main-card-0").src = getCardImage(cards[0]);
    document.getElementById("main-card-1").src = getCardImage(cards[1]);

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
            socket.send(`/throw ${userPlayerId} ${card_idx}`);
        }
    });
}

const initializeDraggableCard = (card: any) => {
    const { socket } = useWebSocket();

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
            powerContainer.innerText = "";
        }
        if (powerState == "ChoosingCardToGive") {
            const card_idx = card.id.split("-")[2];
            socket.send(`/throw_2 ${card_idx}`);
        }
    });
}

const initializeDroppableCard = (card) => {
    const { socket } = useWebSocket();

    card.addEventListener('click', (e) => {
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

    card.addEventListener('dragstart', (e) => {
        e.dataTransfer.setData('text/plain', card.id); // Store dragged card's ID
    });

    card.addEventListener('dragover', (e) => {
        e.preventDefault(); // Allow drop
    });

    card.addEventListener('drop', (e) => {
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
