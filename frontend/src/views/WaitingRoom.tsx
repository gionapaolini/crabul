import { AnimatedList } from "@/components/ui/animated-list";
import ShineBorder from "@/components/ui/shine-border";
import React, { useEffect, useState } from "react";

const WaitingRoom = () => {
  const [players, setPlayers] = useState<string[]>([]);
  const [isGameReady, setGameReady] = useState(false);
  const [countdown, setCountdown] = useState<number | null>(null);

  useEffect(() => {
    console.log("countdown", countdown);
    if (countdown !== null) {
      const timer = setTimeout(() => {
        if (countdown > 1) {
          setCountdown(countdown - 1);
        } else {
          setCountdown(null);
        }
      }, 1500); // Change number every 1.5 seconds
      return () => clearTimeout(timer);
    }
  }, [countdown]);

  useEffect(() => {
    const timeout1 = setTimeout(() => {
      setPlayers((prev) => [...prev, "Giona"]);
    }, 2000);

    const timeout2 = setTimeout(() => {
      setPlayers((prev) => [...prev, "Mario"]);
    }, 3200);

    return () => {
      clearTimeout(timeout1);
      clearTimeout(timeout2);
    };
  }, []);

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
              {!isGameReady ? (
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
                    Giangi
                    <span className="text-black text-2xl">(you)</span>
                  </div>

                  <>
                    {isGameReady ? (
                      <h3 className="w-fit text-green-400 p-2 font-game text-2xl">
                        Ready
                      </h3>
                    ) : (
                      <button
                        onClick={() => {
                          setGameReady(true);
                          setCountdown(3);
                        }}
                        className="btn-game w-fit text-white rounded-lg p-2 font-game text-xl"
                      >
                        I'm Ready
                      </button>
                    )}
                  </>
                </div>
                {players.map((player, index) => (
                  <React.Fragment key={index}>
                    <div className="font-game text-white text-4xl flex justify-between items-center">
                      <div className="flex items-center">{player}</div>
                      <div></div>
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
