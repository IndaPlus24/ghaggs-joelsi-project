use strum_macros::EnumIter;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq)]
pub enum Suit {
    Spades,
    Clubs,
    Hearts,
    Diamonds,
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq)]
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
pub enum GameState {
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