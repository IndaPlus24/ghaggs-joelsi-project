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