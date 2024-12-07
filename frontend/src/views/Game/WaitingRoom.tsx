import { AnimatedList } from "@/components/ui/animated-list";
import ShineBorder from "@/components/ui/shine-border";
import { useRoomStore } from "@/store/roomStore";
import { useSocketStore } from "@/store/socketStore";
import React, { useEffect, useState } from "react";
import { useNavigate } from "react-router";

const WaitingRoom = () => {
  const navigate = useNavigate();
  const myPlayerName = useRoomStore((state) => state.myPlayerName);
  const myPlayerId = useRoomStore((state) => state.myPlayerId);
  const players = useRoomStore((state) => state.players_list);
  const roomId = useRoomStore((state) => state.roomId);
  const handleWebSocketMessage = useSocketStore(
    (state) => state.handleWebSocketMessage
  );
  const socket = useSocketStore((state) => state.socket);
  const message = useSocketStore((state) => state.message);
  const connect = useSocketStore((state) => state.connect);
  const navigation = useSocketStore((state) => state.navigation);

  const [countdown, setCountdown] = useState<number | null>(null);

  useEffect(() => {
    if (navigation) {
      navigate(navigation);
    }
  }, [navigation]);

  useEffect(() => {
    if (myPlayerName == "") {
      navigate("/", { replace: true });
    } else {
      const endpoint = roomId?.trim() ? `connect/${roomId}` : "connect";
      connect(endpoint);
    }
  }, []);

  useEffect(() => {
    if (!message) return;

    try {
      console.log(message);
      handleWebSocketMessage(message);
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
          socket.send("/start");
          navigate("/play");
          setCountdown(null);
        }
      }, 1000);
      return () => clearTimeout(timer);
    }
  }, [countdown]);

  return (
    <>
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
                  {Object.entries(players).map(([id, name], index) => (
                    <React.Fragment key={index}>
                      <div className="font-game text-white text-4xl flex justify-between items-center">
                        <div className="flex items-center">
                          {name}
                          {myPlayerId == id && (
                            <span className="text-black text-2xl">
                              &nbsp;(you)
                            </span>
                          )}
                        </div>

                        <div>
                          {/* {player.isReady && (
                              <h3 className="w-fit text-green-400 p-2 font-game text-2xl">
                                Ready
                              </h3>
                            )} */}
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
          disabled={Object.entries(players).length == 0}
          onClick={() => {
            setCountdown(3);
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
