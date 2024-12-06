import { WebSocketProvider } from "@/context/WebSocketContext";
import { useRoomStore } from "@/store/roomStore";
import { Navigate, Outlet } from "react-router";

const GameLayout = () => {
  const playerName = useRoomStore((state) => state.myPlayerName);

  if (!playerName) {
    return <Navigate to="/" replace />;
  }

  return (
    <WebSocketProvider initialName={playerName}>
      <Outlet />
    </WebSocketProvider>
  );
};

export default GameLayout;
