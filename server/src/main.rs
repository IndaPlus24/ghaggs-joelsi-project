use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures_util::{SinkExt, StreamExt};
use poker::Game;
use poker::structs::enums::{
    ClientMessage, GamePhase, GameStateInfo, PlayerPublicInfo, ServerMessage,
};
use poker::structs::pot::Pot;
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

        // DEBUG
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
                g.player_has_acted.push(false);
                g.pot = Pot::new(g.players.len());
            }

            clients.lock().unwrap().insert(
                addr,
                PlayerConnection {
                    id: player_id,
                    name: player_name.clone(),
                    addr,
                },
            );

            let _ = write
                .send(Message::Text(
                    serde_json::to_string(&ServerMessage::Welcome(player_name.clone(), player_id))
                        .unwrap()
                        .into(),
                ))
                .await;

            loop {
                tokio::select! {
                    Some(Ok(msg)) = read.next() => {
                        if let Message::Text(text) = msg {
                            if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                                println!("{} sent: {:?}", player_name, client_msg);

                                let mut g = game.lock().unwrap();
                                let action_approved = match client_msg {
                                    ClientMessage::Join(name) => {
                                        player_name = name.clone();
                                        if player_id >= g.players.len() {
                                            println!("Invalid player_id: {}, skipping Join", player_id);
                                            return;
                                        }

                                        g.players[player_id].name = name.clone();
                                        println!("{} has joined with played id: {}", name, player_id);

                                        if g.players.len() >= 2 && g.phase == GamePhase::Waiting {
                                            g.start_game();
                                            println!("Game started!");
                                        }
                                        true
                                    },
                                    ClientMessage::Bet(amount) => {
                                        match g.bet(player_id, amount){
                                            Ok(()) => {
                                                g.mark_acted(player_id);
                                                println!("Player id: {} raised by {}", player_id, amount);
                                                true
                                            }
                                            Err(e) => {
                                                let _ = tx.send(ServerMessage::Error(e.to_string()));
                                                false
                                            }
                                        }
                                    },
                                    ClientMessage::Fold => {
                                        let _ = g.fold(player_id);
                                        g.mark_acted(player_id);
                                        println!("Player id: {} folded", player_id);
                                        true
                                    },
                                    ClientMessage::Check => {
                                        match g.check(player_id) {
                                            Ok(()) => {
                                                g.mark_acted(player_id);
                                                println!("Player id: {} checked", player_id);
                                                true
                                            }
                                            Err(e) => {
                                            let _ = tx.send(ServerMessage::Error(e.to_string()));
                                            false
                                            }
                                        }
                                    },
                                    ClientMessage::Call => {
                                        match g.call(player_id) {
                                            Ok(()) => {
                                                g.mark_acted(player_id);
                                                println!("Player id: {} called the raise", player_id);
                                                true
                                            }
                                            Err(e) => {
                                                let _ = tx.send(ServerMessage::Error(e.to_string()));
                                                false
                                            }
                                        }
                                    },
                                };

                                if !action_approved {
                                    continue;
                                }

                                let alive: Vec<usize> = g.players
                                .iter()
                                .enumerate()
                                .filter(|(_, p)| !p.is_folded)
                                .map(|(i, _)| i)
                                .collect();

                            if alive.len() == 1 {
                                let winner = alive[0];
                                g.phase = GamePhase::Showdown;
                                g.winner = Some(winner);
                                let _ = tx.send(ServerMessage::GameState(build_state(&g, Some(winner))));
                                let game_clone = game.clone();
                                let tx_clone = tx.clone();
                                tokio::spawn(async move {
                                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                                    let mut g = game_clone.lock().unwrap();
                                    g.start_game();
                                    let fresh = build_state(&g, None);
                                    let _ = tx_clone.send(ServerMessage::GameState(fresh));
                                });
                                continue;
                            }

                                let n = g.players.len();
                                let mut next = (g.current_turn_index + 1) % n;
                                while g.players[next].is_folded {
                                    next = (next + 1) % n;
                                }
                                g.current_turn_index = next;

                                let all_acted = g.all_acted();
                                let matched = g.non_folded_players_match_bet();
                                if all_acted && matched {
                                    g.advance_phase();
                                    let winner = if g.phase == GamePhase::Showdown { Some(g.best_hand()) } else { None };
                                    if winner != None {
                                        println!("Player {:?} won", winner);
                                    }
                                    let _ = tx.send(ServerMessage::GameState(build_state(&g, winner)));
                                }
                                else {
                                    let _ = tx.send(ServerMessage::GameState(build_state(&g, None)));
                                }

                                if g.phase == GamePhase::Showdown {
                                    let game_clone = game.clone();
                                    let tx_clone = tx.clone();
                                    tokio::spawn(async move {
                                        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                                        let mut g = game_clone.lock().unwrap();
                                        g.start_game();

                                        let fresh_state = build_state(&g, None);
                                        let _ = tx_clone.send(ServerMessage::GameState(fresh_state));
                                    });
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

fn build_state(game: &Game, winner: Option<usize>) -> GameStateInfo {
    let players = game
        .players
        .iter()
        .enumerate()
        .map(|(i, p)| PlayerPublicInfo {
            id: i,
            name: format!("Player{}", i),
            chips: p.chips.clone(),
            is_folded: p.is_folded,
            hand: if p.hand.cards.len() == 2 {
                Some([p.hand.cards[0].clone(), p.hand.cards[1].clone()])
            } else {
                None
            },
        })
        .collect();

    GameStateInfo {
        pot: game.pot.total,
        players,
        board: game.board.clone(),
        current_turn: game.current_turn_index,
        phase: game.phase,
        winner,
    }
}
