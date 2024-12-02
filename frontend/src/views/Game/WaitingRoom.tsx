import { AnimatedList } from "@/components/ui/animated-list";
import ShineBorder from "@/components/ui/shine-border";
import { useWebSocket } from "@/context/WebSocketContext";
import { getSocketMessage } from "@/lib/websocket.utils";
import { WebSocketDataType } from "@/models/WebSocketDataType";
import React, { useEffect, useState } from "react";
import { Location, useLocation, useNavigate } from "react-router";

const WaitingRoom = () => {
  const navigate = useNavigate();

  const location: Location & {
    state: {
      playerName: string;
      roomCode?: string;
    };
  } = useLocation();

  const { connect, message, isConnected } = useWebSocket();

  const [players, setPlayers] = useState<
    { name: string; id: string; isReady: boolean }[]
  >([]);
  const [roomId, setRoomId] = useState<number | null>(null);
  const [myPlayerId, setMyPlayerId] = useState<string | null>(null);

  useEffect(() => {
    if (!location.state) {
      navigate("/", { replace: true });
    }

    if (location.state?.playerName?.trim()) {
      if (location.state?.roomCode?.trim()) {
        // connect to existing room server
        connect(`connect/${location.state.roomCode}`);
        return;
      }
      // connect to new room server
      connect("connect");
    }
  }, []);

  useEffect(() => {
    if (message) {
      const playerAdded = getSocketMessage(
        WebSocketDataType.PlayerJoined,
        message
      );

      const playerLeft: number = getSocketMessage(
        WebSocketDataType.PlayerLeft,
        message
      );

      if (playerAdded) {
        const { player_id, player_name, room_id, player_list } = playerAdded;
        setRoomId(room_id);
        if (player_name === location.state.playerName) {
          setMyPlayerId(player_id);
        }

        const playersArray = Object.entries<any>(player_list).map(
          ([id, name]) => ({
            id,
            name,
            isReady: false,
          })
        );

        setPlayers(playersArray);
      }

      if (playerLeft) {
        setPlayers((prev) =>
          prev.filter((player) => +player.id !== +playerLeft)
        );
      }
    }
  }, [message]);

  // TODO: later, needs player ready state
  const [countdown, setCountdown] = useState<number | null>(null);
  useEffect(() => {
    if (countdown !== null) {
      const timer = setTimeout(() => {
        if (countdown > 0) {
          setCountdown(countdown - 1);
        } else {
          setCountdown(null);
        }
      }, 1500); // Change number every 1.5 seconds
      return () => clearTimeout(timer);
    }
  }, [countdown]);

  // Se tutti i giocatori sono pronti, inizia il countdown
  useEffect(() => {
    if (players.length > 0) {
      const allPlayersReady = players.every((player) => player.isReady);
      if (allPlayersReady) {
        countdown == null && setCountdown(3);
        countdown == 0 && navigate("/"); // procede con il gioco
      } else {
        setCountdown(null);
      }
    }
    // Tiene traccia del countdown. Se entrano altri giocatori mentre il countdown è in corso, lo interrompe
  }, [players, countdown]);

  const setPlayerReady = (id: string) => {
    setPlayers((prev) =>
      prev.map((player) =>
        player.id === id ? { ...player, isReady: true } : player
      )
    );
  };

  // END later

  return (
    <>
      <main
        className="min-h-screen flex flex-col bg-center bg-cover"
        style={{ backgroundImage: "url(sfondo-pattern.jpg)" }}
      >
        <div
          className="min-h-screen flex flex-col bg-center items-center md:!bg-contain bg-no-repeat"
          style={{ backgroundImage: "url(sfondo.png)", backgroundSize: "150%" }}
        >
          <img
            src="crabul_logo.png"
            className="w-full max-w-48 mx-auto"
            alt=""
          />

          <section>
            {roomId && (
              <div className="text-center text-white font-game text-4xl my-8">
                <h2>Room</h2>
                <span className="text-5xl">{roomId}</span>
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
                  {myPlayerId && (
                    <>
                      <div className="font-game text-white text-4xl flex justify-between items-center">
                        <div className="flex items-center">
                          {
                            players.find((player) => +player.id == +myPlayerId)
                              ?.name
                          }
                          <span className="text-black text-2xl">
                            &nbsp;(you)
                          </span>
                        </div>
                      </div>

                      {players
                        .filter((player) => +player.id != +myPlayerId)
                        .map((player, index) => (
                          <React.Fragment key={index}>
                            <div className="font-game text-white text-4xl flex justify-between items-center">
                              <div className="flex items-center">
                                {player.name}
                              </div>

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
              disabled={players.length <= 1}
              className="disabled:opacity-50 btn-game text-4xl font-game text-white w-full p-4 mt-4"
            >
              Start Game
            </button>
          </section>
        </div>
      </main>

      {countdown !== null && (
        <div className="absolute bg-black inset-0 bg-opacity-20 font-game text-crab text-[10rem] flex justify-center items-center">
          <div className="countdown-number">{countdown}</div>
        </div>
      )}
    </>
  );
};

export default WaitingRoom;
