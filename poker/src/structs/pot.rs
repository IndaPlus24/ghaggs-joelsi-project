use std::vec;

#[derive(Debug)]
pub struct Pot {
    pub total: u32,
    pub contributions: Vec<u32>,
    pub current_bet: u32,
    pub player_bets: Vec<u32>,
}

impl Pot {
    // Initialise the pot
    pub fn new(players: usize) -> Self {
        Pot {
            total: 0,
            contributions: vec![0; players], // No one has any contribution in the start
            current_bet: 0,
            player_bets: vec![0; players],
        }
    }
    // Add players contribution to the pot
    pub fn add_constribution(&mut self, player_index: usize, amount: u32) {
        if player_index < self.contributions.len() {
            self.contributions[player_index] += amount;
            self.player_bets[player_index] += amount;
            self.total += amount;

            if self.player_bets[player_index] > self.current_bet {
                self.current_bet = self.player_bets[player_index];
            }
        }
    }

    // Copy the amount of chips that were contributed to then be able to add contribution to the pot
    pub fn get_player_contribution(&self, player_index: usize) -> u32 {
        self.contributions.get(player_index).copied().unwrap_or(0)
    }

    // Reset bets so every possible playeraction is available in the start of the next round
    pub fn reset_round(&mut self) {
        self.current_bet = 0;
        self.player_bets = vec![0; self.player_bets.len()];
    }

    // Reset pot after a round
    pub fn reset(&mut self) {
        self.total = 0;
        self.contributions = vec![0; self.contributions.len()];
        self.reset_round();
    }
}