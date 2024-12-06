import { Toaster } from "@/components/ui/toaster";
import { WebSocketProvider } from "@/context/WebSocketContext";
import { useRoomStore } from "@/store/roomStore";
import { Outlet } from "react-router";

const GameLayout = () => {
  const playerName = useRoomStore((state) => state.myPlayerName);

  // if (!playerName) {
  //   return <Navigate to="/" replace />;
  // }

  return (
    <WebSocketProvider initialName={playerName}>
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

          <Outlet />
        </div>
      </main>
      <Toaster />
    </WebSocketProvider>
  );
};

export default GameLayout;
