use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use shared::{ServerMessage, ClientMessage};
use futures_util::{StreamExt, SinkExt};
use poker::structs::player::Player;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:9001").await.unwrap();
    println!("Server running on ws://127.0.0.1:9001");

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(async move {
            let ws_stream = accept_async(stream).await.unwrap();
            println!("Client connected from: {}", addr);

            let (mut write, mut read) = ws_stream.split();

            while let Some(Ok(msg)) = read.next().await {
                if let Message::Text(text) = msg {
                    let client_msg: ClientMessage = serde_json::from_str(&text).unwrap();
                    println!("Received from client: {:?}", client_msg);

                    let reply = ServerMessage::Welcome("Welcome to the poker table!".to_string());
                    let reply_json = serde_json::to_string(&reply).unwrap();
                    write.send(Message::Text(reply_json.into())).await.unwrap();
                }
            }

            println!("Client disconnected");
        });
    }
}