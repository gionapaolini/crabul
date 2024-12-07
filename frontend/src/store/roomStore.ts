import { PlayerJoinedRes } from "@/models/WebSocketDataType";
import { create } from "zustand";

interface RoomState {
    players_list: Record<string, string>;
    roomId: string | null;
    myPlayerId: string | null;
    myPlayerName: string;
}

interface RoomActions {
    updatePlayers: (playerList: Record<string, string>) => void;
    handlePlayerJoined: (payload: PlayerJoinedRes) => void;
    handlePlayerLeft: (playerId: number) => void;
    setRoomId: (roomId: string) => void;
    setMyPlayerId: (myPlayerId: string) => void;
    setMyPlayerName: (myPlayerName: string) => void;
}

export const useRoomStore = create<RoomState & RoomActions>((set, get) => ({
    players_list: {}, // same as the api
    roomId: null,
    myPlayerId: null,
    myPlayerName: "",
    updatePlayers: (playerList: Record<string, string>) => set({ players_list: playerList }),
    handlePlayerJoined: (payload: PlayerJoinedRes) => {
        const { player_id, player_name, room_id, player_list } = payload;

        set({
            roomId: room_id.toString(),
            myPlayerId: player_name == get().myPlayerName ? player_id.toString() : get().myPlayerId,
        })

        get().updatePlayers(player_list);
    },
    handlePlayerLeft: (playerLeftId: number) => {
        const newList = { ...get().players_list };
        delete newList[playerLeftId.toString()];
        set({ players_list: newList });
    },
    setRoomId: (roomId: string) => set({ roomId }),
    setMyPlayerId: (myPlayerId: string) => set({ myPlayerId }),
    setMyPlayerName: (myPlayerName: string) => set({ myPlayerName }),
}));
