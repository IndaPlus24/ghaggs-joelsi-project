use super::card::Card;
use super::enums::{Rank, Suit};
use rand::seq::SliceRandom;
use strum::IntoEnumIterator;

pub struct Deck {
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let mut deck: Vec<Card> = Vec::new();
        for suit in Suit::iter() {
            for rank in Rank::iter() {
                let card: Card = Card{ suit: suit, rank: rank };
                deck.push(card);
            }
        }
        Deck{cards: deck}
    }

    /// Shuffles deck randomly.
    /// 
    /// Behavior:
    /// - Takes the cards currently in the deck and puts them in a random order.
    /// - Will NOT reinitialize the deck to 52 cards. This can be done with the Deck.reset() function.
    pub fn shuffle(&mut self) {
        let mut rng = rand::rng();
        self.cards.shuffle(&mut rng);
    }

    /// Draws the requested amount of cards and returns them as a Vector.
    /// 
    /// # Parameters
    /// - 'amount': usize that dictates how many cards will be drawn
    /// 
    /// # Returns
    /// - A Result<Vec<Card>, &´static str> where:
    ///     - Vec<Card>: is a vector containing the cards that were drawn from the deck.
    ///     - &´static string: is the error in case there are less cards in the deck than what was requested to draw.
    /// 
    /// Example:
    /// '''
    /// let mut deck = Deck::new();
    ///
    /// match deck.draw(5) {
    ///     Ok(drawn_cards) => {
    ///         println!("You drew the following cards:");
    ///         for card in drawn_cards {
    ///             println!("{:?}", card);
    ///         }
    ///     }
    ///     Err(err) => {
    ///         println!("Error drawing cards: {}", err);
    ///     }
    /// }
    /// '''
    pub fn draw(&mut self, amount: usize) -> Result<Vec<Card>, &'static str> {
        if self.cards.len() < amount {
            return Err("Not enough cards in deck");
        }
        let drawn_cards: Vec<Card> = (0..amount)
            .filter_map(|_| self.cards.pop())
            .collect();
        Ok(drawn_cards)
    }

    /// Resets the deck, making it sorted (like buying a new playing card deck).
    pub fn reset(&mut self) {
        let mut deck: Vec<Card> = Vec::new();
        for suit in Suit::iter() {
            for rank in Rank::iter() {
                let card: Card = Card{ suit: suit, rank: rank };
                deck.push(card);
            }
        }
        self.cards = deck;
    }
}