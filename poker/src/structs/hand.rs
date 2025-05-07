use super::card::Card;
use super::collection::Collection;
use super::misc::rank_to_words;
use poker_eval::eval::five::{build_tables as build_tables_five, get_rank_five, TableFive};
use poker_eval::eval::seven::{build_tables as build_tables_seven, get_rank as get_rank_seven, TableSeven};
use std::sync::Arc;
use itertools::Itertools;

#[derive(Clone)]
pub struct Hand {
    pub cards: Vec<Card>,
}

impl Hand {
    pub fn new() -> Self {
        Hand { cards: Vec::new() }
    }

    /// Evaluates a hand combined with the board cards
    /// 
    /// # Parameters
    /// - 'board': The cards on the board (flop, turn, river).
    /// - 't5': precalculated rank lookup table for five cards.
    /// - 't7': precalculated rank lookup table for seven cards.
    /// 
    /// # Returns
    /// A tuple (rank, description) where:
    /// - 'rank': u32 from 0-7452 where lower number => better hand
    /// - 'description': &str that contains the rank converted to the actual hand type, e.g. "high card", "one pair",...
    /// 
    /// # Behavior
    /// - Returns (0, "error") if amount of cards is not a real poker hand.
    /// - For five cards (hand + flop), uses t5 to check hand rank.
    /// - For six cards (hand + flop + turn), creates all possible 5 combinations, evaluates them by t5, and returns the best rank.
    /// - For seven cards (hand + flop + turn + river), uses t7 to check hand rank.
    /// 
    /// # Example
    /// '''
    /// let hand_rank: (u32, &str) = game.players[0].hand.evaluate(&game.board, &game.t5, &game.t7);
    /// println!("Hand rank: {}, Hand type: {}", hand_rank.0, hand_rank.1);
    /// '''
    pub fn evaluate(&self, board: &Vec<Card>, t5: &TableFive, t7: &Arc<TableSeven>) -> (u32, &str) {
        let mut cards = Collection::new();
        cards.cards.extend(&self.cards);
        cards.cards.extend(board);
        let amount_of_cards: usize = &self.cards.len() + board.len();

        if amount_of_cards < 5 || amount_of_cards > 7 {
            return (0, "Wrong amount of cards in hand + board in evaluate()");
        }

        if amount_of_cards == 5 {
            let card_vector: Vec<usize> = cards.cards
                .into_iter()
                .map(|card| card.as_index())
                .collect::<Vec<usize>>();

                match card_vector.try_into() {
                    Ok(card_values) => {
                        let hand_rank: u32 = get_rank_five(&t5, card_values);
                        return (hand_rank, rank_to_words(hand_rank))
                    },
                    Err(_) => return (0, "Failed to convert to 5-card array in evaluate()"),
                }
        }
        else if amount_of_cards == 6 {
            let hand_rank: Option<u32> = cards.cards
                .into_iter()
                .map(|card| card.as_index())
                .collect::<Vec<usize>>()
                .into_iter()
                .combinations(5)
                .map(|combination| {
                    match combination.try_into() {
                        Ok(array) => Some(get_rank_five(&t5, array)),
                        Err(_) => None,
                    }
                })
                .flatten()
                .max();

            match hand_rank {
                Some(rank) => (rank, rank_to_words(rank)),
                None => (0, "Failed to evaluate 6-card hand in evaluate()")
            };
        }
        else if amount_of_cards == 7 {
            let card_vector: Vec<usize> = cards.cards
                .into_iter()
                .map(|card| card.as_index())
                .collect();
        
            match card_vector.try_into() {
                Ok(card_values) => {
                    let hand_rank: u32 = get_rank_seven(&t7, card_values);
                    return (hand_rank, rank_to_words(hand_rank))
                },
                Err(_) => return (0, "Failed to convert to 7-card array in evaluate()"),
            }
        }
        else {
            println!("How did we get here...?"); // If you know, you know
        }
        (0, "ERROR")
    }
}