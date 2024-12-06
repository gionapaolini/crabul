import { createContext, PropsWithChildren, useCallback, useState } from "react";

export const WebSocketContext = createContext<{
  socket: any;
  message: any;
  isConnected: boolean;
  connect: (endpoint: string) => void;
} | null>(null);

export const WebSocketProvider = ({
  children,
  initialName,
}: PropsWithChildren & {
  initialName: string;
}) => {
  const [socket, setSocket] = useState<any>(null);
  const [message, setMessage] = useState<any>();
  const [isConnected, setIsConnected] = useState<boolean>(false);

  const connect = useCallback(
    (endpoint: string) => {
      const { location } = window;
      const proto = location.protocol.startsWith("https") ? "wss" : "ws";
      const host = "49.13.158.245:5000"; // location.host;
      const wsUri = `${proto}://${host}/${endpoint}?name=${initialName}`;

      const ws = new WebSocket(wsUri);

      ws.onopen = () => {
        setIsConnected(true);
      };

      ws.onmessage = (ev) => {
        const msg = JSON.parse(ev.data);
        setMessage(msg);
      };

      ws.onclose = () => {
        setIsConnected(false);
        setSocket(null);
      };

      setSocket(ws);
    },
    [initialName]
  );

  return (
    <WebSocketContext.Provider
      value={{
        socket,
        message,
        isConnected,
        connect,
      }}
    >
      {children}
    </WebSocketContext.Provider>
  );
};
