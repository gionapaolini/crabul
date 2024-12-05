const GameRoom = () => {
  return (
    // TODO Revisionare template
    <div id="game-room" style={{ display: "none" }}>
      <div className="flex flex-col items-center h-screen bg-gray-800 text-black">
        <div className="">
          <picture>
            <source
              srcSet="crabul_logo.png"
              media="(prefers-color-scheme: dark)"
            />
            <img
              src="crabul_logo.png"
              alt="Crabul Logo"
              className="img-fluid"
              style={{ width: "150px", height: "150px;" }}
            />
          </picture>
        </div>
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
              <button disabled id="draw-card-btn" className="btn btn-primary">
                Draw Card
              </button>
              <button disabled id="crabul-btn" className="btn btn-primary">
                Go Crabul
              </button>
            </div>

            {/* <!-- Main Player Card Section --> */}
            <div className="col">
              <div className="p-3 rounded-lg">
                <div className="font-bold text-center text-white">
                  <span id="power-container"></span>
                  <button
                    id="end-turn-button"
                    className="btn btn-primary"
                    style={{ display: "none", marginLeft: "20px" }}
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
  );
};

export default GameRoom;
