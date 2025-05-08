use poker::structs::enums::GamePhase as GameState;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    GameState(GameState),
    Welcome(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Join(String),
    Bet(u32),
    Fold,
    Call,
    Check,
}