import { AnimatedList } from "@/components/ui/animated-list";
import ShineBorder from "@/components/ui/shine-border";
import React, { useEffect, useState } from "react";
import { useNavigate } from "react-router";

const WaitingRoom = () => {
  const navigate = useNavigate();

  const myPlayerId = "0";

  const [players, setPlayers] = useState<
    { name: string; id: string; isReady: boolean }[]
  >([{ name: "Giangi", id: "0", isReady: false }]);

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

  // Aggiunge giocatori fittizi per simulare l'arrivo di altri giocatori
  useEffect(() => {
    const timeout1 = setTimeout(() => {
      setPlayers((prev) => [
        ...prev,
        { name: "Giona", id: "1", isReady: false },
      ]);
    }, 2000);

    const timeout1_2 = setTimeout(() => {
      setPlayerReady("1"); // Giona
    }, 7200);

    const timeout2 = setTimeout(() => {
      setPlayers((prev) => [
        ...prev,
        { name: "Mario", id: "2", isReady: false },
      ]);
    }, 3200);

    const timeout2_2 = setTimeout(() => {
      setPlayerReady("2");
    }, 6200);

    return () => {
      clearTimeout(timeout1);
      clearTimeout(timeout2);
      clearTimeout(timeout1_2);
      clearTimeout(timeout2_2);
    };
  }, []);

  // Se tutti i giocatori sono pronti, inizia il countdown
  useEffect(() => {
    if (players.length > 0) {
      const allPlayersReady = players.every((player) => player.isReady);
      if (allPlayersReady) {
        countdown == null && setCountdown(3);
        countdown == 0 && navigate("/");
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

  return (
    <>
      <main
        className="min-h-screen flex flex-col bg-center bg-cover"
        style={{ backgroundImage: "url(sfondo-pattern.jpg)" }}
      >
        <div
          className="min-h-screen flex flex-col bg-center md:!bg-contain bg-no-repeat"
          style={{ backgroundImage: "url(sfondo.png)", backgroundSize: "150%" }}
        >
          <img
            src="crabul_logo.png"
            className="w-full max-w-48 mx-auto"
            alt=""
          />

          <ShineBorder className="mt-[10vh] w-full max-w-[500px] mx-auto bg-transparent backdrop-blur-sm p-8">
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
                <div className="font-game text-white text-4xl flex justify-between items-center">
                  <div className="flex items-center">
                    {players.find((player) => player.id === myPlayerId)?.name}
                    <span className="text-black text-2xl">(you)</span>
                  </div>

                  <>
                    {players.find((player) => player.id === myPlayerId)
                      ?.isReady ? (
                      <h3 className="w-fit text-green-400 p-2 font-game text-2xl">
                        Ready
                      </h3>
                    ) : (
                      <button
                        onClick={() => {
                          setPlayerReady(myPlayerId);
                        }}
                        className="btn-game w-fit text-white rounded-lg p-2 font-game text-xl"
                      >
                        I'm Ready
                      </button>
                    )}
                  </>
                </div>
                {players
                  .filter((player) => player.id !== myPlayerId)
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
              </AnimatedList>
            </div>
          </ShineBorder>
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
