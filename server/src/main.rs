use std::{collections::HashMap, net::SocketAddr, sync::{Arc, Mutex}};

use futures_util::{SinkExt, StreamExt};
use poker::Game;
use poker::structs::enums::{ClientMessage, ServerMessage, GameStateInfo, PlayerPublicInfo, GamePhase};
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

    let game_for_broadcast = Arc::clone(&game);
    let tx_clone = broadcast_tx.clone();

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let _ = tx_clone.send(ServerMessage::Error("Just testing".to_string()));
            let g = game_for_broadcast.lock().unwrap();
            for (i, _) in g.players.iter().enumerate() {
                let _ = tx_clone.send(ServerMessage::GameState(build_state(&g, i, None)));
            }
        }
    });

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
                serde_json::to_string(&ServerMessage::Welcome(player_name.clone(), player_id)).unwrap().into()
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
                                        if player_id >= g.players.len() {
                                            eprintln!("Invalid player_id: {}, skipping Join", player_id);
                                            return;
                                        }
                                
                                        g.players[player_id].name = name.clone();
                                        println!("{} has joined.", name);
                                
                                        if g.players.len() >= 1 && g.phase == GamePhase::Waiting {
                                            g.start_game();
                                            println!("Game started!");
                                            let _ = tx.send(ServerMessage::GameState(build_state(&g, player_id, None)));
                                        }
                                    },
                                    ClientMessage::Bet(amount) => {
                                        let _ = g.bet(player_id, amount);
                                    },
                                    ClientMessage::Fold => {
                                        g.fold(player_id);
                                    },
                                    ClientMessage::Check => {
                                        let _ = g.check(player_id);
                                    },
                                    ClientMessage::Call => {
                                        let _ = g.call(player_id);
                                    },
                                }

                                // Check for phase progression
                                let all_acted = true; // Replace with real check
                                let matched = g.non_folded_players_match_bet();
                                let mut winner: Option<usize> = None;
                                if all_acted && matched {
                                    g.advance_phase();
                                    winner = if g.phase == GamePhase::Showdown {
                                        Some(g.best_hand())
                                    } else {
                                        None
                                    };
                                }
                                let _ = tx.send(ServerMessage::GameState(build_state(&g, player_id, winner)));
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


fn build_state(game: &Game, viewer_id: usize, winner: Option<usize>) -> GameStateInfo {
    let players = game.players.iter().enumerate().map(|(i, p)| {
        PlayerPublicInfo {
            id: i,
            name: format!("Player{}", i),
            chips: p.chips.chips,
            is_folded: p.is_folded,
            hand: if i == viewer_id || game.phase == GamePhase::Showdown {
                if p.hand.cards.len() == 2 {
                    Some([p.hand.cards[0].clone(), p.hand.cards[1].clone()])
                } else {
                    None
                }
            } else {
                None
            }
        }
    }).collect();

    GameStateInfo {
        pot: game.pot.total,
        players,
        board: game.board.clone(),
        current_turn: 0,
        phase: format!("{:?}", game.phase),
        winner,
    }
}