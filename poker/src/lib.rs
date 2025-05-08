pub mod structs;

use poker_eval; // https://docs.rs/poker_eval/latest/poker_eval/

use std::sync::Arc;

use poker_eval::eval::five::{build_tables as build_tables_five, TableFive};
use poker_eval::eval::seven::{build_tables as build_tables_seven, TableSeven};

use structs::{
    deck::Deck,
    card::Card,
    player::Player,
    pot::Pot,
    enums::GamePhase,
};

///// TODO: FUNKTION SOM JÄMFÖR ALLAS HÄNDER I GAME-STRUCTEN!!!

pub type Board = Vec<Card>;

pub struct Game {
    pub deck: Deck,
    pub players: Vec<Player>,
    pub board: Board,
    pub t5: TableFive,
    pub t7: Arc<TableSeven>,
    pub pot: Pot,
    pub phase: GamePhase,
    pub winner: Option<usize>,
    pub current_turn_index: usize,
    pub player_has_acted: Vec<bool>,
}

impl Game {
    pub fn new(players: usize, initial_chips: u32) -> Self {
        let mut player_list: Vec<Player> = Vec::new();
        for _ in 0..players {
            player_list.push(Player::new(initial_chips)); // Each player starts with initial chips
        }

        let pot = Pot::new(players); // Initialise a pot with number of players

        Game { deck: Deck::new(), players: player_list, board: Vec::new(), t5: build_tables_five(false), t7: build_tables_seven(false), pot, phase: GamePhase::Waiting, winner: None, current_turn_index: 0, player_has_acted: vec![false; players] }
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
        self.pot.current_bet = amount;
        self.player_has_acted = vec![false; self.players.len()];
        self.player_has_acted[player_index] = true;

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

    // Award the pot to the winner of the round
    pub fn award_pot_to_winner(&mut self) {
        let winner_index = self.best_hand();
        let winnings = self.pot.total;
        self.players[winner_index].chips.add(winnings);
        self.pot.reset();
    }

    // Reset pot after a round
    pub fn reset_round(&mut self) {
        self.pot.reset_round();
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

    pub fn advance_phase(&mut self) {
        self.player_has_acted = vec![false; self.players.len()];

        match self.phase {
            GamePhase::Waiting => {
                // Do nothing
            }
            GamePhase::Preflop => {
                if let Ok(flop) = self.deck.draw(3) {
                    self.board.extend(flop);
                }
                self.phase = GamePhase::Flop;
            }
            GamePhase::Flop => {
                if let Ok(turn) = self.deck.draw(1) {
                    self.board.extend(turn);
                }
                self.phase = GamePhase::Turn;
            }
            GamePhase::Turn => {
                if let Ok(river) = self.deck.draw(1) {
                    self.board.extend(river);
                }
                self.phase = GamePhase::River;
            }
            GamePhase::River => {
                self.phase = GamePhase::Showdown;
            }
            GamePhase::Showdown => {
                self.award_pot_to_winner();
                self.reset_round();
                self.board.clear();
                self.phase = GamePhase::Preflop;

                for player in &mut self.players {
                    player.is_folded = false;
                }
                for player in &mut self.players {
                    if let Ok(cards) = self.deck.draw(2) {
                        player.hand.cards = cards;
                    }
                }
            }
        }
    }

    pub fn start_game(&mut self) {
        self.player_has_acted = vec![false; self.players.len()];
        self.deck.shuffle();
        self.board.clear();
        self.pot.total = 0;
        self.pot.current_bet = 0;
        self.pot.player_bets = vec![0; self.players.len()];
        self.phase = GamePhase::Preflop;
        self.winner = None;
    
        for player in self.players.iter_mut() {
            player.is_folded = false;
            player.hand.cards.clear();
        }
    
        for player in self.players.iter_mut() {
            if let Ok(cards) = self.deck.draw(2) {
                player.hand.cards = cards;
            }
        }
    
        self.current_turn_index = 0;
    }
    
    pub fn mark_acted(&mut self, player_index: usize) {
        self.player_has_acted[player_index] = true;
    }

    pub fn all_acted(&self) -> bool {
        self.players
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.is_folded)
            .all(|(i, _)| self.player_has_acted[i])
    }
}