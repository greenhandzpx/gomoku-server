use std::{env, io::Error};

// use futures_util::{future, StreamExt, TryStreamExt, SinkExt};
use game::{Player, check_waiting_player};
use log::{info, debug};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{tungstenite::handshake::server::{Request, Response}, accept_hdr_async};


mod game;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = env_logger::try_init();
    let addr = env::args().nth(1).unwrap_or_else(|| "localhost:12345".to_string());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream) {

    let callback = |req: &Request, mut response: Response| {
        info!("Received a new ws handshake");
        info!("The request's path is: {}", req.uri().path());
        info!("The request's headers are:");
        for (ref header, _value) in req.headers() {
            debug!("* {}: {:?}", header, _value);
        }

        let headers = response.headers_mut();
        headers.append("Access-Control-Allow-Origin", "*".parse().unwrap());

        Ok(response)
    };
    let ws_stream = accept_hdr_async(stream, callback)
        .await
        .expect("Error during the websocket handshake occurred");

    let player = Player::new(ws_stream);
    check_waiting_player(player).await;

    // while let Some(msg) = ws_stream.next().await {
    //     let msg = msg.unwrap();
    //     if msg.is_text() || msg.is_binary() {
    //         println!("Server on message: {:?}", &msg);
    //         ws_stream.send(msg).await.unwrap();
    //     }
    // }

}