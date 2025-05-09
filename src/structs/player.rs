use super::hand::Hand;
use super::playerchips::PlayerChips;

#[derive(Clone)]
pub struct Player {
    pub hand: Hand,
    pub chips: PlayerChips,
    pub is_folded: bool,
    pub is_all_in: bool,
}
impl Player {
    pub fn new(initial_chips: u32) -> Self {
        Player { 
            hand: Hand::new(),
            chips: PlayerChips::new(initial_chips),
            is_folded: false,
            is_all_in: false,
        }
    }
}