// Funktioner:
// Skapa_lek()
// Blanda_lek()
// Evaluera_hand()
// Ge()

use poker_eval; // https://docs.rs/poker_eval/latest/poker_eval/

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

impl Rank {
    pub fn to_char() -> char {
        match self {
            Ace => 'A',
            Two => '2',
            Three => '3',
            Four => '4',
            Five => '5',
            Six => '6',
            Seven => '7',
            Eight => '8',
            Nine => '9',
            Ten => 'T',
            Jack => 'J',
            Queen => 'Q',
            King => 'K',
        }
    }
}