import { PlayerJoinedRes } from "@/models/WebSocketDataType";
import { Player } from "@/views/Game/WaitingRoom";
import { create } from "zustand";

interface RoomState {
    players: Player[];
    players_list: any;
    roomId: number | null;
    myPlayerId: number | null;
}

interface RoomActions {
    updatePlayers: (playerList: Record<string, string>) => void;
    handlePlayerJoined: (payload: PlayerJoinedRes, myPlayerName: string) => void;
    handlePlayerLeft: (playerId: number) => void;
}

export const useRoomStore = create<RoomState & RoomActions>((set, get) => ({
    players_list: {}, // same as the api
    players: [], // manipulated
    roomId: null,
    myPlayerId: null,

    updatePlayers: (playerList: Record<string, string>) => {
        const players = Object.entries(playerList).map(
            ([id, name]): Player => ({
                id,
                name,
                isReady: false,
            })
        );

        set({ players_list: playerList });
        set({ players });
    },
    handlePlayerJoined: (payload: PlayerJoinedRes, myPlayerName: string) => {
        const { player_id, player_name, room_id, player_list } = payload;
        set({
            roomId: room_id,
            myPlayerId: player_name === myPlayerName ? player_id : get().myPlayerId,
        });
        get().updatePlayers(player_list);
    },
    handlePlayerLeft: (playerId: number) => {
        set((state) => ({ players: state.players.filter((p) => +p.id !== playerId) }));
    }
}));
