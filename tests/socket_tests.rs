use std::{net::TcpListener, time::Duration};

use crabul::{api::run, room::events::RoomEvent};
use futures_util::StreamExt;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let server = run(listener).expect("Failed to bind address");
    tokio::spawn(server);
    format!("127.0.0.1:{}", port)
}

#[tokio::test]
async fn new_game_room() {
    let address = spawn_app();

    let (mut ws_stream, _) = connect_async(&format!("ws://{address}/connect?name=gio"))
        .await
        .unwrap();

    let received = ws_stream.next().await.unwrap().unwrap();
    if let Message::Text(payload) = received {
        serde_json::from_str::<RoomEvent>(&payload).unwrap();
    } else {
        panic!("Failed to get payload")
    }
}

#[tokio::test]
async fn join_game_room() {
    let address = spawn_app();

    let (mut ws_stream1, _) = connect_async(&format!("ws://{address}/connect?name=gio"))
        .await
        .unwrap();

    let received = timeout(Duration::from_millis(1), ws_stream1.next()).await.unwrap().unwrap().unwrap();
    // let received = ws_stream1.next().now_or_never().unwrap().unwrap().unwrap();
    let event = match received {
        Message::Text(payload) => serde_json::from_str::<RoomEvent>(&payload).unwrap(),
        _ => panic!("Error when reading ws msg"),
    };
    let room_id = match event {
        RoomEvent::PlayerJoined { room_id, .. } => room_id,
        _ => panic!("Wrong event received"),
    };

    let (mut ws_stream2, _) = connect_async(&format!("ws://{address}/connect/{room_id}?name=gioggi"))
        .await
        .unwrap();

    let received = timeout(Duration::from_millis(1), ws_stream1.next()).await.unwrap().unwrap().unwrap();
    match received {
        Message::Text(payload) => serde_json::from_str::<RoomEvent>(&payload).unwrap(),
        _ => panic!("Error when reading ws msg"),
    };
    let received = timeout(Duration::from_millis(1), ws_stream2.next()).await.unwrap().unwrap().unwrap();
    match received {
        Message::Text(payload) => serde_json::from_str::<RoomEvent>(&payload).unwrap(),
        _ => panic!("Wrong event"),
    };
}
