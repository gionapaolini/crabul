use std::net::TcpListener;

use crabul::api::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000").expect("Failed to bind random port");
    run(listener)?.await
}
