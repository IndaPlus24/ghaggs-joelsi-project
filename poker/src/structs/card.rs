use super::enums::{Rank, Suit};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}
impl Card {
    pub fn as_index(self) -> usize {
        let suit_offset = match self.suit {
            Suit::Clubs => 0,
            Suit::Diamonds => 1,
            Suit::Hearts => 2,
            Suit::Spades => 3,
        };
    
        let rank_offset = match self.rank {
            Rank::Two => 0,
            Rank::Three => 4,
            Rank::Four => 8,
            Rank::Five => 12,
            Rank::Six => 16,
            Rank::Seven => 20,
            Rank::Eight => 24,
            Rank::Nine => 28,
            Rank::Ten => 32,
            Rank::Jack => 36,
            Rank::Queen => 40,
            Rank::King => 44,
            Rank::Ace => 48,
        };
    
        suit_offset + rank_offset
    }
}