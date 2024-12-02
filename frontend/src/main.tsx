import { createRoot } from "react-dom/client";
import { BrowserRouter, Route, Routes } from "react-router";
import "./index.css";
import StartGame from "./views/StartGame.tsx";
import { GameRoutes } from "./routes/game.routes.tsx";

createRoot(document.getElementById("root")!).render(
  <BrowserRouter>
    <Routes>
      {/* Welcome screen */}
      <Route index element={<StartGame />} />
      {/* Game */}
      <Route path="/*" element={<GameRoutes />} />
    </Routes>
  </BrowserRouter>
);
