import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter, Route, Routes } from "react-router";
import "./index.css";
import StartGame from "./views/StartGame.tsx";
import WaitingRoom from "./views/WaitingRoom.tsx";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <BrowserRouter>
      <Routes>
        <Route index element={<StartGame />} />
        <Route path="waiting-room" element={<WaitingRoom />} />
      </Routes>
    </BrowserRouter>
  </StrictMode>
);
