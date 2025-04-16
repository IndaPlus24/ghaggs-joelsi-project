use poker_eval; // https://docs.rs/poker_eval/latest/poker_eval/
use rand::Rng;
use rand::seq::SliceRandom;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use itertools::Itertools;

use poker_eval::eval::five::{build_tables as build_tables_five, get_rank_five};
use poker_eval::eval::seven::{build_tables as build_tables_seven, get_rank as get_rank_seven};


#[derive(Clone, Copy, Debug)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

pub struct Collection {
    pub cards: Vec<Card>,
}

pub struct Hand {
    pub cards: Vec<Card>,
}
pub struct Deck {
    pub cards: Vec<Card>,
}
pub type Board = Vec<Card>;

pub struct Player {
    pub hand: Hand,
}

pub struct Game {
    pub deck: Deck,
    pub players: Vec<Player>,
    pub board: Board,
}

impl Game {
    pub fn new(players: usize) -> Self {
        let mut player_list: Vec<Player> = Vec::new();
        for i in 0..players {
            player_list.push(Player::new());
        }
        Game { deck: Deck::new(), players: player_list, board: Vec::new() }
    }
}

impl Player {
    pub fn new() -> Self {
        Player { hand: Hand::new() }
    }
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
        let mut rng = rand::rng();
        self.cards.shuffle(&mut rng);
    }

    pub fn draw(&mut self, amount: usize) -> Vec<Card> {
        let mut drawn_cards: Vec<Card> = Vec::new();
        for _ in 0..amount {
            drawn_cards.push(self.cards.pop().expect("Card draw failed in fn draw()"));
        }
        drawn_cards
    }
}

impl Hand {
    pub fn new() -> Self {
        Hand { cards: Vec::new() }
    }


    pub fn evaluate(&self, board: &Vec<Card>) -> (u32, &str) {
        let mut cards = Collection::new();
        cards.cards.extend(&self.cards);
        cards.cards.extend(board);
        let amount_of_cards: usize = &self.cards.len() + board.len();

        if amount_of_cards < 5 || amount_of_cards > 7 {
            return (0, "ERROR");
        }

        if amount_of_cards == 5 {
            let t5 = build_tables_five(true);
            let card_values: [usize; 5] = cards.cards
                .into_iter()
                .map(|card| card.as_index())
                .collect::<Vec<usize>>()
                .try_into()
                .expect("Couldn't convert Vec to arr for five cards in evaluate()");
            let hand_rank: u32 = get_rank_five(&t5, card_values);
            return (hand_rank, rank_to_words(hand_rank))
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
                .expect("No max returned for 6 card hand in evaluate()");

            return (hand_rank, rank_to_words(hand_rank))
        }
        else if amount_of_cards == 7 {
            let t7 = build_tables_seven(false);
            let card_values: [usize; 7] = cards.cards
                .into_iter()
                .map(|card| card.as_index())
                .collect::<Vec<usize>>()
                .try_into()
                .expect("Couldn't convert Vec to arr for seven cards in evaluate()");
            let hand_rank: u32 = get_rank_seven(&t7, card_values);
            return (hand_rank, rank_to_words(hand_rank))
        }
        else {
            println!("How did we get here...?");
        }
        (0, "ERROR")
    }
}

impl Collection {
    pub fn new() -> Self {
        Collection { cards: Vec::new() }
    }
}

pub fn rank_to_words(rank: u32) -> &'static str {
    match rank {
        7452..=7461 => "Straight Flush",
        7296..=7451 => "Four of a Kind",
        7140..=7295 => "Full House",
        5863..=7139 => "Flush",
        5853..=5862 => "Straight",
        4995..=5852 => "Three of a Kind",
        4137..=4994 => "Two Pair",
        1277..=4136 => "One Pair",
        0..=1276    => "High Card",
        _ => "Unknown Hand",
    }
}