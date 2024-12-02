import { WebSocketProvider } from "@/context/WebSocketContext";
import { PropsWithChildren } from "react";

const GameWrapper = ({
  children,
  initialName,
}: PropsWithChildren & { initialName: string }) => {
  return (
    <WebSocketProvider initialName={initialName}>{children}</WebSocketProvider>
  );
};

export default GameWrapper;
