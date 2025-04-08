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

impl PlayingCard {
    fn as_index(self) -> usize {
        let suit_offset = match self.suit {
            Suit::Spades => 0,
            Suit::Hearts => 13,
            Suit::Diamonds => 26,
            Suit::Clubs => 39,
        };
    
        let rank_offset = match self.rank {
            Rank::Two => 0,
            Rank::Three => 1,
            Rank::Four => 2,
            Rank::Five => 3,
            Rank::Six => 4,
            Rank::Seven => 5,
            Rank::Eight => 6,
            Rank::Nine => 7,
            Rank::Ten => 8,
            Rank::Jack => 9,
            Rank::Queen => 10,
            Rank::King => 11,
            Rank::Ace => 12,
        };
    
        suit_offset + rank_offset
    }
}