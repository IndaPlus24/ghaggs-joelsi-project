use std::{collections::HashMap, net::SocketAddr, sync::{Arc, Mutex}};

use futures_util::{SinkExt, StreamExt};
use poker::Game;
use poker::structs::enums::{ClientMessage, ServerMessage, GameStateInfo, PlayerPublicInfo};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpListener, sync::broadcast};
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[derive(Debug)]
struct PlayerConnection {
    id: usize,
    name: String,
    addr: SocketAddr,
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:9001").await.unwrap();
    println!("Server running on ws://127.0.0.1:9001");

    let game = Arc::new(Mutex::new(Game::new(0, 1000)));
    let (broadcast_tx, _) = broadcast::channel::<ServerMessage>(32);
    let clients = Arc::new(Mutex::new(HashMap::new()));
    let mut player_counter = 0;

    while let Ok((stream, addr)) = listener.accept().await {
        let ws_stream = accept_async(stream).await.unwrap();
        println!("Client connected from: {}", addr);

        let game = Arc::clone(&game);
        let tx = broadcast_tx.clone();
        let mut rx = tx.subscribe();
        let clients = Arc::clone(&clients);

        let player_id = player_counter;
        player_counter += 1;

        tokio::spawn(async move {
            let (mut write, mut read) = ws_stream.split();

            let mut player_name = format!("Player{player_id}");
            {
                let mut g = game.lock().unwrap();
                g.players.push(poker::structs::player::Player::new(1000));
            }

            clients.lock().unwrap().insert(addr, PlayerConnection {
                id: player_id,
                name: player_name.clone(),
                addr,
            });

            let _ = write.send(Message::Text(
                serde_json::to_string(&ServerMessage::Welcome(player_name.clone())).unwrap().into()
            )).await;

            loop {
                tokio::select! {
                    Some(Ok(msg)) = read.next() => {
                        if let Message::Text(text) = msg {
                            if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                                println!("{} sent: {:?}", player_name, client_msg);

                                let mut g = game.lock().unwrap();
                                match client_msg {
                                    ClientMessage::Join(name) => {
                                        player_name = name.clone();
                                        tx.send(ServerMessage::GameState(build_state(&g, player_id))).unwrap();
                                    }
                                    ClientMessage::Bet(amount) => {
                                        if g.bet(player_id, amount).is_ok() {
                                            tx.send(ServerMessage::GameState(build_state(&g, player_id))).unwrap();
                                        }
                                    }
                                    ClientMessage::Fold => {
                                        g.fold(player_id);
                                        tx.send(ServerMessage::GameState(build_state(&g, player_id))).unwrap();
                                    }
                                    ClientMessage::Check => {
                                        if g.check(player_id).is_ok() {
                                            tx.send(ServerMessage::GameState(build_state(&g, player_id))).unwrap();
                                        }
                                    }
                                    ClientMessage::Call => {
                                        if g.call(player_id).is_ok() {
                                            tx.send(ServerMessage::GameState(build_state(&g, player_id))).unwrap();
                                        }
                                    }
                                }

                                // Check for phase progression
                                let all_acted = true; // Replace with real check
                                let matched = g.non_folded_players_match_bet();

                                if all_acted && matched {
                                    g.advance_phase();
                                    tx.send(ServerMessage::GameState(build_state(&g, player_id))).unwrap();
                                }
                            }
                        }
                    }
                    Ok(msg) = rx.recv() => {
                        let _ = write.send(Message::Text(serde_json::to_string(&msg).unwrap().into())).await;
                    }
                    else => break
                }
            }

            println!("{} disconnected", player_name);
            clients.lock().unwrap().remove(&addr);
        });
    }
}

fn build_state(game: &Game, viewer_id: usize) -> GameStateInfo {
    let players = game.players.iter().enumerate().map(|(i, p)| {
        PlayerPublicInfo {
            id: i,
            name: format!("Player{}", i),
            chips: p.chips.chips,
            is_folded: p.is_folded,
            hand: if i == viewer_id || game.board.len() == 5 {
                Some([p.hand.cards[0].clone(), p.hand.cards[1].clone()])
            } else {
                None
            },
        }
    }).collect();

    GameStateInfo {
        pot: game.pot.total,
        players,
        board: game.board.clone(),
        current_turn: 0,
        phase: format!("{:?}", game.phase),
        winner: None,
    }
}
