use super::card::Card;

pub struct Collection {
    pub cards: Vec<Card>,
}

impl Collection {
    pub fn new() -> Self {
        Collection { cards: Vec::new() }
    }
}