# poker_backend

## Features
Contains structs and functions to easily allow the construction of a simple poker game.
Uses the crate [poker_eval](https://docs.rs/poker_eval/latest/poker_eval/) to evaluate hands based on some bit shifting black magic.

## Usage

### Importing the library
---
If *lib.rs* is in the same directory as *main.rs*, the module and its necessary functions can be imported with the **mod** and **use** keywords.
```
mod lib;
use lib::{Game, Player, Deck, Card, Suit, Rank, rank_to_words}; // Some structs and functions will be unneccessary to import.
```

### Initializing
---
The library contains a *Game* struct. This contains all the necessary fields to play a game of poker.
To initialize the *Game*, create a new *Game* and pass the amount of players as parameter.
**!!!** *Remember*: the deck is sorted by default. Before dealing, shuffle the deck!
```
let mut game = Game::new(5); // Creates a new game with five players.
game.deck.shuffle();         // Sorts the playing deck.
```

### Poker activities
---
Here are some examples of functions that will be used during a game of poker.
```
// Draw hands for all players
for player in &mut game.players {
    match game.deck.draw(2) {
        Ok(cards) => player.hand.cards = cards,
        Err(e) => println!("Error drawing cards: {}", e),
    }
}

// Draw flop
match game.deck.draw(3) {
    Ok(cards) => game.board.extend(cards),
    Err(e) => println!("Error drawing flop: {}", e),
}

// Evaluate a player's hand
let hand_rank = game.players[0].hand.evaluate(&game.board, &game.t5, &game.t7);
println!("Player 1 hand rank: {}, hand type: {}", hand_rank.0, hand_rank.1);
```

### Notes on evaluate
---
evaluate() returns a tuple with both the "backend" rank and the "frontend" hand description. To access these, .0 in the tuple is the rank and .1 is the hand description.
Additionaly, evaluate() must always be passed *&Game.t5* and *&Game.t7* as parameters. This is due to the heavy computational load required to create these variables. They are hand lookup tables, and recalculating them on every hand evaluation would render the game very slow.