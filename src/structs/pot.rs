#[derive(Debug)]
pub struct Pot {
    pub total: u32,
    pub contributions: Vec<u32>
}

impl Pot {
    // Initialise the pot
    pub fn new(players: usize) -> Self {
        Pot {
            total: 0,
            contributions: vec![0; players] // No one has any contribution in the start
        }
    }
    // Add players contribution to the pot
    pub fn add_constribution(&mut self, player_index: usize, amount: u32) {
        if player_index < self.contributions.len() {
            self.contributions[player_index] += amount;
            self.total += amount;
        }
    }
    // Reset pot after a round
    pub fn reset(&mut self) {
        self.total = 0;
        self.contributions = vec![0; self.contributions.len()];
    }

    pub fn get_player_contribution(&self, player_index: usize) -> u32 {
        self.contributions.get(player_index).copied().unwrap_or(0)
    }
}