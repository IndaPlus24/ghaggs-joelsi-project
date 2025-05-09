pub mod structs;

use poker_eval;
use structs::hand::Hand; // https://docs.rs/poker_eval/latest/poker_eval/

use std::sync::Arc;

use poker_eval::eval::five::{build_tables as build_tables_five, get_rank_five, TableFive};
use poker_eval::eval::seven::{build_tables as build_tables_seven, get_rank as get_rank_seven, TableSeven};

use structs::deck::Deck;
use structs::card::Card;
use structs::player::{self, Player};
use structs::pot::Pot;

///// TODO: FUNKTION SOM JÄMFÖR ALLAS HÄNDER I GAME-STRUCTEN!!!

pub type Board = Vec<Card>;

pub struct Game {
    pub deck: Deck,
    pub players: Vec<Player>,
    pub board: Board,
    pub t5: TableFive,
    pub t7: Arc<TableSeven>,
    pub pot: Pot,
}

impl Game {
    pub fn new(players: usize, initial_chips: u32) -> Self {
        let mut player_list: Vec<Player> = Vec::new();
        for _ in 0..players {
            player_list.push(Player::new(initial_chips)); // Each player starts with initial chips
        }

        let pot = Pot::new(players); // Initialise a pot with number of players

        Game { deck: Deck::new(), players: player_list, board: Vec::new(), t5: build_tables_five(false), t7: build_tables_seven(false), pot }
    }

    // Handling playeraction: betting
    pub fn bet(&mut self, player_index: usize, amount: u32) -> Result<(), &'static str> {
        let player = &mut self.players[player_index];
        // Check if player has enough chips
        if player.chips.chips < amount {
            return Err("Player doesn't have enough chips");
        }

        let current_bet = self.pot.current_bet;

        // If player tries to bet under the current bet
        if amount < current_bet {
            return Err("Bet must be at least the the current bet(or use call)");
        }

        // Deduct the amount of chips from the betted player and add it to the pot
        player.chips.deduct(amount);
        self.pot.add_constribution(player_index, amount);

        Ok(())
    }

    // Handling playeraction: calling
    pub fn call(&mut self, player_index: usize) -> Result<(), &'static str> {
        let player = &mut self.players[player_index];
        let to_call = self.pot.current_bet.saturating_sub(self.pot.player_bets[player_index]);

        // Incase a player can't call because of lack of chips. Use all in instead
        if player.chips.chips < to_call {
            return Err("Not enough chips to call");
        }

        // Deduct the amount of chips from the calling player and add it to the pot
        player.chips.deduct(to_call);
        self.pot.add_constribution(player_index, to_call);

        Ok(())
    }

    // Handling playeraction: checking
    pub fn check(&self, player_index: usize) -> Result<(), &'static str> {
        // If a player has betted, checks are invalid
        if self.pot.current_bet > self.pot.player_bets[player_index] {
            return Err("Cannot check: need to call, raise or fold");
        }
        Ok(())
    }

    // Handling playeraction: folding
    pub fn fold(&mut self, player_index: usize) {
        self.players[player_index].is_folded = true;
    }

    pub fn all_in(&mut self, player_index: usize) -> Result<(), &'static str> {
        let player = &mut self.players[player_index];

        // Player must have chips to go all in
        if player.chips.chips == 0 {
            return Err("Player has no chips to go all-in")
        }

        let all_in_amount = player.chips.chips;

        // Deduct all the chips from the player and add it to the pot
        player.chips.deduct(all_in_amount);
        self.pot.add_constribution(player_index, all_in_amount);

        self.pot.current_bet = self.pot.current_bet.max(all_in_amount);

        player.is_all_in = true;

        self.pot.player_bets[player_index] = all_in_amount;
        Ok(())
    }

    // Check how many players that haven't folded, true or false.
    pub fn non_folded_players_match_bet(&self) -> bool {
        for (i, player) in self.players.iter().enumerate() {
            // If player has folded, skip their turn
            if player.is_folded {
                continue;
            }
            let player_bet = self.pot.player_bets[i];
            if player_bet != self.pot.current_bet {
                return false;
            }
        }
        true
    }

    pub fn award_pot_to_specific_player(&mut self, winner_index: usize) {
        let winnings = self.pot.total;
        self.players[winner_index].chips.chips += winnings;
        self.pot.total = 0;
    }

    // Award the pot to the winner of the round
    pub fn award_pot_to_winner(&mut self) {
        let winner_index = self.best_hand();
        let winnings = self.pot.total;
        self.players[winner_index].chips.add(winnings);
        self.pot.reset();
    }

    pub fn reset_pot(&mut self) {
        self.pot.reset_round();
    }

    // Reset round
    pub fn reset_round(&mut self) {
        self.pot.reset_round();
        self.board.clear(); // Clear community cards

        // Reset each player's hand and folded status
        for player in &mut self.players {
            player.hand = Hand::new();
            player.is_folded = false;
        }

        self.deck = Deck::new();
        self.deck.shuffle();
    }

    // Reset the game after a player wins to be able to play again if wanted
    pub fn reset_game(&mut self) {
        self.pot.reset();
        for player in &mut self.players {
            player.chips.chips = 1000;
            player.is_folded = false;
        }
    }

    /// Evaluates all players hands and returns the index of the player with the winning hand.
    /// 
    /// Returns:
    ///  - usize: contains the index of the player with the best hand in Game.players.
    pub fn best_hand(&self) -> usize {
        let mut vec: Vec<(u32, usize)> = Vec::new();
        for (i, player) in self.players.iter().enumerate() {
            if player.is_folded {
                continue;
            }
            let (rank, _) = player.hand.evaluate(&self.board, &self.t5, &self.t7);
            vec.push((rank, i));
        }
        vec.sort_by_key(|&(rank, _)| std::cmp::Reverse(rank));
        vec[0].1
    }
}