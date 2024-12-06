import { createRoot } from "react-dom/client";
import { BrowserRouter, Route, Routes } from "react-router";
import "./index.css";
import GameLayout from "./routes/gameLayout.tsx";
import WaitingRoom from "./views/Game/WaitingRoom.tsx";
import StartGame from "./views/StartGame.tsx";
import GameRoom from "./views/Game/GameRoom.tsx";

createRoot(document.getElementById("root")!).render(
  <BrowserRouter>
    <Routes>
      {/* Welcome screen */}
      <Route index element={<StartGame />} />
      {/* Game */}
      <Route element={<GameLayout />}>
        <Route path="waiting-room" element={<WaitingRoom />} />
        <Route path="play" element={<GameRoom />} />
      </Route>
    </Routes>
  </BrowserRouter>
);
