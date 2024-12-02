import { WebSocketDataType } from "@/models/WebSocketDataType";

export const getSocketMessage = (key: WebSocketDataType, message: any) => {
    if (key in message) {
        return message[key];
    }
};