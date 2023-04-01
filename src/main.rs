use std::{env, io::Error};

use futures_util::{future, StreamExt, TryStreamExt, SinkExt};
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
    let mut ws_stream = accept_hdr_async(stream, callback)
        .await
        .expect("Error during the websocket handshake occurred");

    while let Some(msg) = ws_stream.next().await {
        let msg = msg.unwrap();
        if msg.is_text() || msg.is_binary() {
            println!("Server on message: {:?}", &msg);
            ws_stream.send(msg).await.unwrap();
        }
    }

    // let addr = stream.peer_addr().expect("connected streams should have a peer address");
    // info!("Peer address: {}", addr);
    // println!("Peer address: {}", addr);

    // let ws_stream = tokio_tungstenite::accept_async(stream)
    //     .await
    //     .expect("Error during the websocket handshake occurred");

    // info!("New WebSocket connection: {}", addr);

    // let (write, read) = ws_stream.split();
    // // We should not forward messages other than text or binary.
    // read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
    //     .forward(write)
    //     .await
    //     .expect("Failed to forward messages")
}