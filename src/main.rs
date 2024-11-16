
use actix_files as fs;
use actix_web::{
    get, rt,
    web::{self},
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use crabul::{
    consts::{PlayerName, RoomId},
    server::{Server, ServerCommander},
    ws_client::WsClient,
};
use serde::Deserialize;
use tokio::spawn;

#[derive(Deserialize)]
struct NameInfo {
    name: PlayerName,
}

#[get("/connect")]
async fn new_room(
    req: HttpRequest,
    stream: web::Payload,
    server_commander: web::Data<ServerCommander>,
    name_info: web::Query<NameInfo>,
) -> Result<HttpResponse, Error> {
    let (res, session, stream) = actix_ws::handle(&req, stream)?;
    let stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    let room_commander = server_commander.new_room().await;
    let (player_id, player_channel) = room_commander
        .new_player(name_info.name.clone())
        .await
        .unwrap();

    let client = WsClient::new(player_id, room_commander, player_channel, stream, session);

    rt::spawn(client.run());

    Ok(res)
}

#[get("/connect/{room_id}")]
async fn join_room(
    req: HttpRequest,
    stream: web::Payload,
    server_commander: web::Data<ServerCommander>,
    name_info: web::Query<NameInfo>,
    path: web::Path<RoomId>,
) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;
    let stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    let room_id = path.into_inner();
    match server_commander.join_room(room_id).await {
        Ok(room_commander) => match room_commander.new_player(name_info.name.clone()).await {
            Ok((player_id, player_channel)) => {
                let client =
                    WsClient::new(player_id, room_commander, player_channel, stream, session);

                rt::spawn(client.run());
            }
            Err(err) => {
                let _ = session.text(serde_json::to_string(&err).unwrap()).await;
                let _ = session.close(None).await;
            }
        },
        Err(err) => {
            let _ = session.text(serde_json::to_string(&err).unwrap()).await;
            let _ = session.close(None).await;
        }
    }

    Ok(res)
}

// async fn index() -> impl Responder {
//     NamedFile::open_async("./index.html").await.unwrap()
// }
 

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (game_server, server_commander) = Server::new();

    spawn(game_server.run());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server_commander.clone()))
            .service(new_room)
            .service(join_room)
            .service(fs::Files::new("/", "static").index_file("index.html"))

    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
