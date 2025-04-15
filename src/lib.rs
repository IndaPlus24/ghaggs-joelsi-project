// Funktioner:
// Skapa_lek()
// Blanda_lek()
// Evaluera_hand()
// Ge()

use poker_eval; // https://docs.rs/poker_eval/latest/poker_eval/
use rand::Rng;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use itertools::Itertools;

use poker_eval::eval::five::{build_tables as build_tables_five, get_rank_five};
use poker_eval::eval::seven::{build_tables as build_tables_seven, get_rank as get_rank_seven};

#[derive(Clone, Copy, Debug)]
pub struct Card {
    suit: Suit,
    rank: Rank,
}

pub struct Collection {
    cards: Vec<Card>,
}

pub struct Hand {
    cards: Vec<Card>,
}
pub struct Deck {
    cards: Vec<Card>,
}
pub struct Board {
    cards: Vec<Card>,
}

pub struct Player {
    cards: Hand,
}

pub struct Game {
    deck: Deck,
    players: Vec<Player>,
}

#[derive(Clone, Copy, Debug, EnumIter)]
pub enum Suit {
    Spades,
    Clubs,
    Hearts,
    Diamonds,
}

#[derive(Clone, Copy, Debug, EnumIter)]
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

impl Card {
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


impl Deck {
    fn new() -> Self {
        let mut deck: Vec<Card> = Vec::new();
        for suit in Suit::iter() {
            for rank in Rank::iter() {
                let card: Card = Card{ suit: suit, rank: rank };
                deck.push(card);
            }
        }
        Deck{cards: deck}
    }

    pub fn shuffle(&mut self) {
        let mut shuffled: Vec<Card> = Vec::new();
        let mut rng = rand::rng();
        let indexes: Vec<usize> = (0..52).collect();
        for i in 0..52 {
            let random_index = rng.random_range(0..indexes.len());
            shuffled[random_index] = self.cards[52 - i];
            self.cards.pop();
        }
        self.cards = shuffled;
    }

    pub fn draw(&mut self, amount: i32) -> Vec<Card> {
        let mut drawn_cards: Vec<Card> = Vec::new();
        for _ in 0..amount {
            drawn_cards.push(self.cards.pop().expect("Card draw failed in fn draw()"));
        }
        drawn_cards
    }
}

impl Hand {
    pub fn evaluate(&self, board: Vec<Card>) -> u32 {
        let mut cards = Collection::new();
        cards.cards.extend(&self.cards);
        let amount_of_cards: usize = &self.cards.len() + board.len();

        if amount_of_cards < 5 || amount_of_cards > 7 {
            return 1
        }

        if amount_of_cards == 5 {
            let t5 = build_tables_five(false);
            let card_values: [usize; 5] = cards.cards.into_iter().map(|card| card.as_index()).collect::<Vec<usize>>().try_into().unwrap();
            let hand_rank: u32 = get_rank_five(&t5, card_values);
            return hand_rank
        }
        else if amount_of_cards == 6 {
            let t5 = build_tables_five(false);
            let hand_rank: u32 = cards.cards
                .into_iter()
                .map(|card| card.as_index())
                .collect::<Vec<usize>>()
                .into_iter()
                .combinations(5)
                .map(|combination| {
                    let array: [usize; 5] = combination.try_into().unwrap();
                    get_rank_five(&t5, array)
                })
                .max()
                .unwrap();

            return hand_rank
        }
        else if amount_of_cards == 7 {
            let t7 = build_tables_seven(false);
            let card_values: [usize; 7] = cards.cards.into_iter().map(|card| card.as_index()).collect::<Vec<usize>>().try_into().unwrap();
            let hand_rank: u32 = get_rank_seven(&t7, card_values);
            return hand_rank
        }
        else {
            println!("How did we get here...?");
        }
        0
    }
}

impl Collection {
    pub fn new() -> Self {
        return Collection { cards: Vec::new() }
    }
}