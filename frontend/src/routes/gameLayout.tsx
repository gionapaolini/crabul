import { WebSocketProvider } from "@/context/WebSocketContext";
import { Navigate, Outlet, useLocation, useParams } from "react-router";

const GameLayout = () => {
  const { name } = useParams();
  const { state } = useLocation();

  const initialName = name || state?.playerName;

  if (!initialName) {
    return <Navigate to="/" replace />;
  }

  return (
    <WebSocketProvider initialName={initialName}>
      <Outlet />
    </WebSocketProvider>
  );
};

export default GameLayout;
