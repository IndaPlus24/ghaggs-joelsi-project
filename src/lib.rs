// Funktioner:
// Skapa_lek()
// Blanda_lek()
// Evaluera_hand()
// Ge()

use poker::{Evaluator, deck, Card}; // From: https://github.com/deus-x-mackina/poker

pub struct PlayingCard {
    suit: Suit,
    rank: Rank,
}

pub enum Suit {
    Spades,
    Clubs,
    Hearts,
    Diamonds,
}

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