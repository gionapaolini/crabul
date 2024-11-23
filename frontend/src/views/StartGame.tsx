import { CoolMode } from "@/components/ui/cool-mode";
import SparklesText from "@/components/ui/sparkles-text";
import { useNavigate } from "react-router";

const StartGame = () => {
  let navigate = useNavigate();

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

          <section className="mt-[10vh]">
            <div className="mx-auto p-5 rounded-lg text-center w-full max-w-[500px]">
              <form className="mb-3 flex flex-col text-2xl p-8 rounded-lg bg-opacity-35">
                <input
                  type="text"
                  id="name"
                  placeholder="Enter your name"
                  className="p-4 bg-white bg-opacity-90 rounded-lg font-game mb-2"
                />
                <button
                  disabled={false} // Change to true to disable the button
                  id="new-room-button"
                  className="btn-game text-white rounded-lg p-2 mt-2 font-game text-3xl"
                  type="button"
                  onClick={() => {
                    navigate("/waiting-room");
                  }}
                >
                  <SparklesText
                    text="Create New Room"
                    className="text-3xl"
                    sparklesCount={4}
                  />
                </button>

                <div className="my-2 font-game text-white">
                  &mdash; or &mdash;
                </div>
                <div className="flex items-stretch w-full">
                  <input
                    id="room-code-input"
                    type="text"
                    className="p-4 bg-white bg-opacity-90 w-full rounded-l-lg font-game"
                    placeholder="Enter room code"
                  />
                  <CoolMode
                    options={{
                      particle: "crabby.png",
                      particleCount: 1,
                      size: 80,
                      speedUp: 25,
                      speedHorz: 10,
                    }}
                  >
                    <button
                      id="join-room-button"
                      type="button"
                      className="btn-game text-white rounded-r-lg p-2 min-w-fit font-game text-3xl"
                    >
                      <SparklesText
                        text="Join Room"
                        className="text-3xl"
                        sparklesCount={4}
                      />
                    </button>
                  </CoolMode>
                </div>
              </form>
            </div>
          </section>
        </div>
      </main>
    </>
  );
};

export default StartGame;
