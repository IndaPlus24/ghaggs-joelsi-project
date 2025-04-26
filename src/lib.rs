pub mod structs;

use poker_eval; // https://docs.rs/poker_eval/latest/poker_eval/

use std::sync::Arc;

use poker_eval::eval::five::{build_tables as build_tables_five, get_rank_five, TableFive};
use poker_eval::eval::seven::{build_tables as build_tables_seven, get_rank as get_rank_seven, TableSeven};

use structs::deck::Deck;
use structs::card::Card;
use structs::player::Player;
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

    pub fn place_bet(&mut self, player_index: usize, bet_amount: u32) -> Result<(), &'static str> {
        if player_index >= self.players.len() {
            return Err("Invalid player index");
        }

        // Check if player has enough chips
        if self.players[player_index].chips.chips < bet_amount {
            return Err("Player doesn't have enough chips");
        }

        // Deduct the chips from the player
        self.players[player_index].chips.deduct(bet_amount);

        // Add to the pot
        self.pot.add_constribution(player_index, bet_amount);

        Ok(())
    }

    pub fn reset_round(&mut self) {
        self.pot.reset();
        for player in &mut self.players {
            player.chips.chips = 1000; // Reset player chips. Just nu kan vi bara recetta tillbaka till start, men vi lär ändra detta när man kan förlora
        }
    }

    /// Evaluates all players hands and returns the index of the player with the winning hand.
    /// 
    /// Returns:
    ///  - usize: contains the index of the player with the best hand in Game.players.
    pub fn best_hand(&self) -> usize {
        let mut vec: Vec<(u32, usize)> = Vec::new();
        for (i, player) in self.players.iter().enumerate() {
            let (rank, _) = player.hand.evaluate(&self.board, &self.t5, &self.t7);
            vec.push((rank, i));
        }
        vec.sort_by_key(|&(rank, _)| std::cmp::Reverse(rank));
        vec[0].1
    }
}