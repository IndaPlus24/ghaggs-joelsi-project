use strum_macros::EnumIter;
use serde::{Serialize, Deserialize};
use super::card::Card;

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq, Serialize, Deserialize)]
pub enum Suit {
    Spades,
    Clubs,
    Hearts,
    Diamonds,
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

impl Rank {
    pub fn to_char(&self) -> char {
        match self {
            Rank::Ace => 'A',
            Rank::Two => '2',
            Rank::Three => '3',
            Rank::Four => '4',
            Rank::Five => '5',
            Rank::Six => '6',
            Rank::Seven => '7',
            Rank::Eight => '8',
            Rank::Nine => '9',
            Rank::Ten => 'T',
            Rank::Jack => 'J',
            Rank::Queen => 'Q',
            Rank::King => 'K',
        }
    }
}

// Texas Hold em
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum GamePhase {
    Preflop, 
    Flop,
    Turn, 
    River,
    Showdown,
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum PlayerActions {
    None,
    Bet,
    Check,
    Call,
    Fold,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    Join(String),
    Bet(u32),
    Fold,
    Check,
    Call,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    Welcome(String),
    GameState(GameStateInfo),
    Error(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameStateInfo {
    pub pot: u32,
    pub players: Vec<PlayerPublicInfo>,
    pub board: Vec<Card>,
    pub current_turn: usize,
    pub phase: String,
    pub winner: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerPublicInfo {
    pub id: usize,
    pub name: String,
    pub chips: u32,
    pub is_folded: bool,
    pub hand: Option<[Card; 2]>, // only shown to the player themselves or on showdown
}