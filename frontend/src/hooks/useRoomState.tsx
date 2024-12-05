import { PlayerJoinedRes } from "@/models/WebSocketDataType";
import { Player } from "@/views/Game/WaitingRoom";
import { useCallback, useState } from "react";

interface RoomState {
  players: Player[];
  roomId: number | null;
  myPlayerId: number | null;
}

export const useRoomState = () => {
  const [state, setState] = useState<RoomState>({
    players: [],
    roomId: null,
    myPlayerId: null,
  });

  const updatePlayers = useCallback((playerList: Record<string, string>) => {
    const players = Object.entries(playerList).map(
      ([id, name]): Player => ({
        id,
        name,
        isReady: false,
      })
    );
    setState((prev) => ({ ...prev, players }));
  }, []);

  const handlePlayerJoined = useCallback(
    (payload: PlayerJoinedRes, myPlayerName: string) => {
      const { player_id, player_name, room_id, player_list } = payload;

      setState((prev) => ({
        ...prev,
        roomId: room_id,
        myPlayerId: player_name === myPlayerName ? player_id : prev.myPlayerId,
      }));

      updatePlayers(player_list);
    },
    []
  );

  const handlePlayerLeft = useCallback((playerId: number) => {
    setState((prev) => ({
      ...prev,
      players: prev.players.filter((p) => +p.id !== playerId),
    }));
  }, []);

  return {
    state,
    handlers: {
      handlePlayerJoined,
      handlePlayerLeft,
    },
  };
};
