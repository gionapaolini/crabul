import { toast } from "@/hooks/use-toast";
import { useGameStore } from "@/store/gameStore";
import { useSocketStore } from "@/store/socketStore";
import { useEffect } from "react";

const GameRoom = () => {
  const notification = useGameStore((state) => state.notifications);

  const discardAction = useGameStore((state) => state.handleDiscardButton);
  const swapAction = useGameStore((state) => state.handleSwapButton);
  const closePeekedCardModalButtonAction = useGameStore(
    (state) => state.handleClosePeekedCardModalButton
  );

  const handleWebSocketMessage = useSocketStore(
    (state) => state.handleWebSocketMessage
  );
  const socketMessage = useSocketStore((state) => state.message);

  const socket = useSocketStore((state) => state.socket);

  useEffect(() => {
    if (!socketMessage) return;

    try {
      handleWebSocketMessage(socketMessage);
    } catch (error) {
      console.error("Error processing websocket message:", error);
    }
  }, [socketMessage]);

  useEffect(() => {
    if (typeof notification == "string") {
      toast({
        description: notification,
      });
    }
  }, [notification]);

  return (
    // TODO Revisionare template
    <>
      <div id="drawn-card-modal" className="custom-modal">
        <div className="custom-modal-content">
          <img
            id="drawn-card"
            src="cards/0_1.svg"
            alt="Card"
            className="img-fluid mb-3"
          />

          <div className="flex justify-around">
            <button
              id="discard-card-btn"
              className="btn btn-danger"
              onClick={discardAction}
            >
              Discard
            </button>

            <button
              id="swap-card-btn"
              className="btn btn-success"
              onClick={swapAction}
            >
              Swap
            </button>
          </div>
        </div>
      </div>

      <div id="peeked-card-modal" className="custom-modal">
        <div className="custom-modal-content">
          <img
            id="peeked-card"
            src="cards/0_1.svg"
            alt="Card"
            className="img-fluid mb-3"
          />
          <div className="flex justify-around">
            <button
              id="close-peeked-card-btn"
              className="btn btn-danger"
              onClick={closePeekedCardModalButtonAction}
            >
              Close
            </button>
          </div>
        </div>
      </div>

      <div
        id="game-result-modal"
        className="custom-modal"
        style={{ display: "none" }}
      >
        <div className="custom-modal-content larger">
          <div id="game-scores"></div>
        </div>
      </div>

      <div id="game-room" className="w-full">
        <div className="w-full flex flex-col items-center h-screen bg-gray-800 text-black">
          <div className="container flex">
            {/* <!-- Left side with stacked player items --> */}
            <div
              id="player-card-container"
              className="w-1/2 flex flex-col gap-2 p-3"
            ></div>

            <div className="w-1/2 flex flex-col justify-content-center items-center">
              <div className="flex gap-4 items-center">
                {/* <!-- Deck image --> */}
                <img
                  id="discard-pile"
                  src="cards/empty.svg"
                  alt="Discard Pile"
                  className="discard-img"
                  style={{ width: "120px" }}
                />

                <img
                  src="cards/deck.svg"
                  alt="Deck"
                  className="img-fluid ms-3"
                  style={{ width: "150px" }}
                />

                {/* <!-- Discard pile image --> */}
              </div>
            </div>
          </div>

          <div className="container mt-4">
            <div className="flex flex-col">
              {/* <!-- Buttons Section (Inline) --> */}
              <div className="flex justify-content-center gap-3 mb-4">
                <button
                  id="draw-card-btn"
                  className="btn btn-game"
                  onClick={() => socket?.send("/draw")}
                >
                  Draw Card
                </button>
                <button
                  id="crabul-btn"
                  className="btn btn-game"
                  onClick={() => socket?.send("/crabul")}
                >
                  Go Crabul
                </button>
              </div>

              {/* <!-- Main Player Card Section --> */}
              <div className="col">
                <div className="p-3 rounded-lg">
                  <div className="font-bold text-center text-white">
                    <span id="power-container"></span>
                    {/* TODO Hidden */}
                    <button
                      id="end-turn-button"
                      className="btn btn-game"
                      onClick={() => socket?.send("/pow4_2 ")}
                    >
                      End Turn
                    </button>
                  </div>

                  <div className="font-bold text-center">Your cards</div>
                  <div
                    id="main-player-card-container"
                    className="flex justify-center gap-4 p-3 mt-2"
                  >
                    {/* <!-- Player cards will appear here --> */}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  );
};

export default GameRoom;
