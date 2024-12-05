
export const createGameHandlers = (setState: React.Dispatch<React.SetStateAction<GameState>>) => ({
    handlePickingPhase: (message: any) => {
        console.log("Picking phase started", message);
        startGame({ cards: message, players: roomState.players });
    },
    handlePlayerTurn: (message: any) => {
        endTurnButton.style.display = "none"; //check a better place to put ths
        if (data["PlayerTurn"] === userPlayerId) {
            createNotification(`Your turn`);
            drawButton.disabled = false;
            crabulButton.disabled = false;
        } else {
            createNotification(`Player turn: ${players[data["PlayerTurn"]]}`);
            drawButton.disabled = true;
            crabulButton.disabled = true;
        }
        coverCards();
        highlightCurrentPlayer(data["PlayerTurn"]);
    },
    handleCardWasDrawn: (message: any) => {
        if (data["CardWasDrawn"] != userPlayerId) {
            createNotification(`Player ${players[data["CardWasDrawn"]]} drew a card from the deck`);
        } else {
            drawButton.disabled = true;
            crabulButton.disabled = true;
        }
    },
    handlePlayerWentCrabul: (message: any) => {
        if (data["PlayerWentCrabul"] != userPlayerId) {
            createNotification(`Player ${players[data["PlayerWentCrabul"]]} went CRABUL!`);
            const badge = document.getElementById(`crabul-badge-${data["PlayerWentCrabul"]}`);
            badge.classList.remove('d-none');
        } else {
            drawButton.disabled = true;
            crabulButton.disabled = true;
        }
    },
    handleDrawnCard: (message: any) => {
        drawnCard.src = getCardImage(data["DrawnCard"]);
        drawnCardModal.style.display = 'flex';
    },
    handleCardSwapped: (message: any) => {
        if (data["CardSwapped"][0] != userPlayerId) {
            createNotification(`Player ${players[data["CardSwapped"][0]]} swapped card ${data["CardSwapped"][1] + 1}`);
        }
    },
    handleCardDiscarded: (message: any) => {
        if (data["CardDiscarded"][0] != userPlayerId) {
            createNotification(`Player ${players[data["CardDiscarded"][0]]} discarded a card`);
        }
        discardPile.src = getCardImage(data["CardDiscarded"][1]);
    },
    handlePowerActivated: (message: any) => {

        if (data["PowerActivated"][0] != userPlayerId) {
            createNotification(
                `Player ${players[data["PowerActivated"][0]]} activated a power: ${data["PowerActivated"][1]}`
            );
        } else {
            powerState = data["PowerActivated"][1];
            powerContainer.innerText = "POWER " + data["PowerActivated"][1] + " ACTIVE!";
        }
    },
    handlePowerUsed: (message: any) => {
        if (data["PowerUsed"][1] != userPlayerId) {
            sendPowerUsedNotification(data["PowerUsed"]);
        } else {
            if (data["PowerUsed"][0] === "CheckAndSwapStage1") {
                powerState = "CheckAndSwapStage2";
                powerContainer.innerText = "SWAP WITH ONE OF YOUR CARD OR END TURN";
                endTurnButton.style.display = 'block';
            } else {
                powerState = null;
                powerContainer.innerText = "";
            }
        }
    },
    handlePeekedCard: (message: any) => {
        peekedCard.src = getCardImage(data["PeekedCard"]);
        peekedCardModal.style.display = 'flex';
    },
    handleGameTerminated: (message: any) => {
        showGameResults(data);
    },
    handleSameCardAttempt: (message: any) => {
        if (data["SameCardAttempt"][4] !== "Success") {
            addCard(data["SameCardAttempt"][0]);
            if (data["SameCardAttempt"][0] in players) {
                createNotification(
                    `${players[data["SameCardAttempt"][0]]} attempted to throw a diplicate unsuccessfully: ${data["SameCardAttempt"][4]}, receiving a penalty card`
                );
            } else {
                createNotification(
                    `You attempted to throw a duplicate unsuccessfully: ${data["SameCardAttempt"][4]}, receive a penalty card`
                );
            }
        } else {
            if (data["SameCardAttempt"][0] in players) {
                createNotification(
                    `Player ${players[data["SameCardAttempt"][0]]} threw a duplicate successfully`);
            } else {
                createNotification(`You threw a duplicate successfully`);
            }

            removeCard(data["SameCardAttempt"][1]);
            if (data["SameCardAttempt"][0] != data["SameCardAttempt"][1]) {
                oldPowerState = powerState;
                oldPowerContainerText = powerContainer.innerText;

                if (data["SameCardAttempt"][0] in players) {
                    powerState = "WaitingForChoosingCardToGive";
                    powerContainer.innerText =
                        `PAUSE! Game will resume after ${data["SameCardAttempt"][0]} choose the card to substitute`;
                } else {
                    powerState = "ChoosingCardToGive";
                    powerContainer.innerText = "PICK ONE OF YOUR CARD TO SUBSTITUTE!";
                }
            }
            discardPile.src = getCardImage(data["SameCardAttempt"][3]);
        }
    },
    handleCardReplaced: (message: any) => {
        if (data["CardReplaced"][0] in players) {
            createNotification(
                `${players[data["CardReplaced"][0]]} have replaced the duplicate with card ${data["CardReplaced"][1]}`
            );
        }
        removeCard(data["CardReplaced"][0]);
        addCard(data["CardReplaced"][2]);
        powerState = oldPowerState;
        powerContainer.innerText = oldPowerContainerText;
    }
})
