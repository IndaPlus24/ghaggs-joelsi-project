use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use url::Url;
use tokio::sync::mpsc::{Receiver, Sender};
use poker::structs::enums::{ClientMessage, ServerMessage};

pub async fn run_client(
    mut to_server_rx: Receiver<ClientMessage>,
    to_game_tx: Sender<ServerMessage>,
) {
    let url = Url::parse("ws://localhost:9001").unwrap();
    let (ws_stream, _) = connect_async(url.to_string()).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    // Task to send messages to the server
    let send_task = tokio::spawn(async move {
        while let Some(msg) = to_server_rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if write.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Task to receive messages from the server
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = read.next().await {
            if let Ok(msg) = serde_json::from_str::<ServerMessage>(&text) {
                if to_game_tx.send(msg).await.is_err() {
                    break;
                }
            }
        }
    });

    // Wait for either task to end
    tokio::select! {
        _ = send_task => println!("Send task ended"),
        _ = recv_task => println!("Receive task ended"),
    }
}
