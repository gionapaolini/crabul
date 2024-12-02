import { Navigate, Route, Routes, useLocation, useParams } from "react-router";
import GameWrapper from "../views/Game/GameWrapper";
import WaitingRoom from "../views/Game/WaitingRoom";

export function GameRoutes() {
  const { name } = useParams();
  const { state } = useLocation();

  const playerName = name || state?.playerName;

  if (!playerName) {
    return <Navigate to="/" replace />;
  }

  return (
    <GameWrapper initialName={playerName}>
      <Routes>
        <Route path="waiting-room" element={<WaitingRoom />} />
      </Routes>
    </GameWrapper>
  );
}
