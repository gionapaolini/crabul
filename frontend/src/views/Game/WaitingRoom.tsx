import { AnimatedList } from "@/components/ui/animated-list";
import ShineBorder from "@/components/ui/shine-border";
import { useWebSocket } from "@/hooks/useWebSocket";
import { getSocketMessage } from "@/lib/websocket.utils";
import { WebSocketDataType } from "@/models/WebSocketDataType";
import { useRoomStore } from "@/store/roomStore";
import React, { useEffect, useState } from "react";
import { useNavigate } from "react-router";

export interface Player {
  id: string;
  name: string;
  isReady: boolean;
}

const WaitingRoom = () => {
  const navigate = useNavigate();
  const { connect, message, socket } = useWebSocket();
  const state = useRoomStore();

  const [countdown, setCountdown] = useState<number | null>(null);

  useEffect(() => {
    if (state.myPlayerName == "") {
      navigate("/", { replace: true });
    } else {
      const endpoint = state.roomId?.trim()
        ? `connect/${state.roomId}`
        : "connect";

      connect(endpoint);
    }
  }, []);

  useEffect(() => {
    if (!message) return;

    try {
      const playerJoined = getSocketMessage(
        WebSocketDataType.PlayerJoined,
        message
      );
      const playerLeft = getSocketMessage(
        WebSocketDataType.PlayerLeft,
        message
      );

      if (playerJoined)
        state.handlePlayerJoined(playerJoined, state.myPlayerName);
      if (playerLeft) state.handlePlayerLeft(playerLeft);
    } catch (error) {
      console.error("Error processing websocket message:", error);
    }
  }, [message]);

  useEffect(() => {
    if (countdown !== null) {
      const timer = setTimeout(() => {
        if (countdown > 0) {
          setCountdown(countdown - 1);
        } else {
          navigate("/play");
          setCountdown(null);
        }
      }, 1000);
      return () => clearTimeout(timer);
    }
  }, [countdown]);

  // TODO: later, needs player ready state

  // // Se tutti i giocatori sono pronti, inizia il countdown
  // useEffect(() => {
  //   if (state.players.length > 0) {
  //     const allPlayersReady = state.players.every((player) => player.isReady);
  //     if (allPlayersReady) {
  //       countdown == null && setCountdown(3);
  //       countdown == 0 && navigate("/"); // procede con il gioco
  //     } else {
  //       setCountdown(null);
  //     }
  //   }
  //   // Tiene traccia del countdown. Se entrano altri giocatori mentre il countdown è in corso, lo interrompe
  // }, [state.players, countdown]);

  // const setPlayerReady = (id: string) => {
  //   setPlayers((prev) =>
  //     prev.map((player) =>
  //       player.id === id ? { ...player, isReady: true } : player
  //     )
  //   );
  // };

  // END later

  return (
    <>
      <section>
        {state.roomId && (
          <div className="text-center text-white font-game text-4xl my-8">
            <h2>Room</h2>
            <span className="text-5xl">{state.roomId}</span>
          </div>
        )}

        <ShineBorder className="w-full max-w-[500px] mx-auto bg-transparent backdrop-blur-sm p-8">
          <section className="font-game text-3xl text-center">
            {!countdown ? (
              <h2 className="flex items-center mb-4">
                Waiting for players
                <span className="loading-dots">
                  <span>.</span>
                  <span>.</span>
                  <span>.</span>
                </span>
              </h2>
            ) : (
              <h2 className="flex items-center mb-4">
                All set! Get ready for the match...
              </h2>
            )}
          </section>

          <div className="w-full max-w-[400px]">
            <AnimatedList className="flex-col-reverse">
              {state.myPlayerId && (
                <>
                  <div className="font-game text-white text-4xl flex justify-between items-center">
                    <div className="flex items-center">
                      {
                        state.players.find(
                          (player) => player.id == state.myPlayerId
                        )?.name
                      }
                      <span className="text-black text-2xl">&nbsp;(you)</span>
                    </div>
                  </div>

                  {state.players
                    .filter((player) => player.id != state.myPlayerId)
                    .map((player, index) => (
                      <React.Fragment key={index}>
                        <div className="font-game text-white text-4xl flex justify-between items-center">
                          <div className="flex items-center">{player.name}</div>

                          <div>
                            {player.isReady && (
                              <h3 className="w-fit text-green-400 p-2 font-game text-2xl">
                                Ready
                              </h3>
                            )}
                          </div>
                        </div>
                      </React.Fragment>
                    ))}
                </>
              )}
            </AnimatedList>
          </div>
        </ShineBorder>

        <button
          disabled={state.players.length <= 1}
          onClick={() => {
            setCountdown(3);
            socket.send("/start");
          }}
          className="disabled:opacity-50 btn-game text-4xl font-game text-white w-full p-4 mt-4"
        >
          Start Game
        </button>
      </section>

      {countdown !== null && (
        <div className="absolute bg-black inset-0 bg-opacity-20 font-game text-crab text-[10rem] flex justify-center items-center">
          <div className="countdown-number">{countdown}</div>
        </div>
      )}
    </>
  );
};

export default WaitingRoom;
