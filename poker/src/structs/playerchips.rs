use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerChips {
    pub chips: u32
}
impl PlayerChips {
    // Initialise a player with a certain amount of chips
    pub fn new(initial_chips: u32) -> Self {
        PlayerChips { chips: initial_chips }
    }
    // Deducting chips from the players "chips-bank", since they're stored
    pub fn deduct(&mut self, amount: u32) -> bool {
        if self.chips >= amount {
            self.chips -= amount;
            true
        }
        else {
            false // Not enough chips
        }
    }
    // Add chips to the players "chips-bank" from the pot when winning a round
    pub fn add(&mut self, amount: u32) {
        self.chips += amount;
    }
}