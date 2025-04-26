use std::{collections::HashMap, vec};

use ggez::{
    glam::Vec2,
    graphics::{self, Color, DrawMode, DrawParam, Image, Mesh, Rect, Text},
    input::{keyboard::{KeyCode, KeyInput}, mouse::MouseButton},
    Context, ContextBuilder, GameResult, event::{self, EventHandler}
};

use ghaggs_joelsi_project::{
    Game,
    structs::{
        player::Player as BackendPlayer,
        enums::{Rank, Suit},
        card::Card
    }
};


fn main() {
    // Make a Context.
    let (mut context, event_loop) = ContextBuilder::new("Poker", "Gustav, Joel")
        .add_resource_path("./resources")
        .build()
        .expect("Failed to create ggez context!");

    let my_game = MyGame::new(&mut context);
    event::run(context, event_loop, my_game);
}

// Texas Hold em
#[derive(Clone, Copy, PartialEq)]
enum GameState {
    Preflop, 
    Flop,
    Turn, 
    River,
    Showdown,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum PlayerActions {
    None,
    Bet,
    Check,
    Call,
    Fold,
}

// Frontend player representation
#[derive(Clone)]
struct FrontendPlayer {
    name: String,
    chips: u32,
    backend_player: BackendPlayer, // Use the backend Player struct    
    position: Vec2
}
struct MyGame {
    card_images: HashMap<String, Image>, // Multiple cards
    players: Vec<FrontendPlayer>,
    chip_image: Image,
    game_state: GameState,
    backend_game: Game, // Backend game logic
    elapsed_time: f32,
    winner_index: Option<usize>,
    player_action: PlayerActions,
    pot: u32, // Get rid of when pot is created in backend
    player_actions_done: Vec<bool>,
    current_player_index: usize 
}

// Helper function to convert backend Card to image key
fn card_to_image_key(card: &Card) -> String {
    let value = match card.rank {
        Rank::Ace => "ace",
        Rank::Two => "2",
        Rank::Three => "3",
        Rank::Four => "4",
        Rank::Five => "5",
        Rank::Six => "6",
        Rank::Seven => "7",
        Rank::Eight => "8",
        Rank::Nine => "9",
        Rank::Ten => "10",
        Rank::Jack => "jack",
        Rank::Queen => "queen",
        Rank::King => "king",
    };

    let suit = match card.suit {
        Suit::Clubs => "clubs",
        Suit::Spades => "spades",
        Suit::Hearts => "hearts",
        Suit::Diamonds => "diamonds",
    };
    
    format!("{}_of_{}", value, suit)
}

fn load_all_cards(context: &mut Context) -> HashMap<String, Image> {
    let suits = ["clubs", "spades", "diamonds", "hearts"];
    let values = ["2", "3", "4", "5", "6", "7", "8", "9", "10", "jack", "queen", "king", "ace"];

    let mut cards = HashMap::new();
    for suit in suits.iter() {
        for value in values.iter() {
            let card_name = format!("{}_of_{}", value, suit);
            let path = format!("/PNG-cards-1.3/{}.png", card_name);
            if let Ok(image) = Image::from_path(context, path) {
                cards.insert(card_name.clone(), image);
            }
            else {
                println!("Could not find card: {}", card_name);
            }
        }
    }
    cards
}
/*
fn generate_deck() -> Vec<String> {
    let suits = ["clubs", "spades", "diamonds", "hearts"];
    let values = ["2", "3", "4", "5", "6", "7", "8", "9", "10", "jack", "queen", "king", "ace"];

    let mut deck = Deck::new();
    for suit in suits.iter() {
        for value in values.iter() {
            deck.push(format!("{}_of_{}", value, suit));
        }
    }
    deck.shuffle(&mut rand::thread_rng());
    deck
}
*/

impl MyGame {
    pub fn new(context: &mut Context) -> MyGame {
        let card_images = load_all_cards(context);
        let chip_image = Image::from_path(context, "/casino-poker-chip-png.webp")
            .expect("Chip image not found");

        let mut backend_game = Game::new(2, 1000);

        backend_game.deck.shuffle();

        let mut frontend_players = vec![
            FrontendPlayer {
                name: "Joel".to_string(),
                chips: 1000,
                backend_player: BackendPlayer::new(1000),
                position: Vec2::new(100.0, 500.0)
            },
            FrontendPlayer {
                name: "Gustav".to_string(),
                chips: 1000,
                backend_player: BackendPlayer::new(1000),
                position: Vec2::new(700.0, 500.0)
            },
        ];

        // Deal cards to players
        for player in 0..backend_game.players.len() {
            if let Ok(cards) = backend_game.deck.draw(2) {
                backend_game.players[player].hand.cards = cards;
            }
        }

        for (i, player) in frontend_players.iter_mut().enumerate() {
            player.backend_player = backend_game.players[i].clone();
        }

        MyGame {
            card_images,
            players: frontend_players,
            chip_image,
            game_state: GameState::Preflop,
            backend_game,
            elapsed_time: 0.0,
            winner_index: None,
            player_action: PlayerActions::None,
            pot: 0,
            player_actions_done: vec![false; 2],
            current_player_index: 0,
        }
    }

    fn sync_pot_and_chips(&mut self) {
        self.pot = self.backend_game.pot.total; // Sync pot with backend
        for (i, player) in self.players.iter_mut().enumerate() {
            player.chips = self.backend_game.players[i].chips.chips; // Sync players with chips
        }
    }

    fn place_bet(&mut self, bet_amount: u32) {
        if let Err(error) = self.backend_game.place_bet(self.current_player_index, bet_amount) {
            println!("Error placing bet: {}", error);
        }
        else {
            // Sync state after placing a bet
            self.sync_pot_and_chips();
        }
    }

    fn reset_game(&mut self) {
        // Reset the game
        self.backend_game = Game::new(2, 1000);
        self.backend_game.deck.shuffle();
        for player in 0..self.backend_game.players.len() {
            if let Ok(cards) = self.backend_game.deck.draw(2) {
                self.backend_game.players[player].hand.cards = cards;
            }
        }
    
        // Sync backend state to frontend state
        for (i, player) in self.players.iter_mut().enumerate() {
            player.backend_player = self.backend_game.players[i].clone();
        }

        // Reset game variables
        self.game_state = GameState::Preflop;
        self.elapsed_time = 0.0;
        self.winner_index = None;
        self.pot = 0;
    }

    fn reset_actions(&mut self) {
        self.player_actions_done = vec![false; self.players.len()];
        self.elapsed_time = 0.0;
        self.player_action = PlayerActions::None;
    }

    fn determine_winner(&self) -> usize {
        self.backend_game.best_hand()
    }
}


impl EventHandler for MyGame {
    fn update(&mut self, context: &mut Context) -> GameResult {
        let delta = ggez::timer::delta(context).as_secs_f32();
        self.elapsed_time += delta;

        let all_acted = self.player_actions_done.iter().all(|&acted| acted);

        // Handle player actions
        match self.player_action {
            PlayerActions::Bet => {
                println!("Player is betting");
                self.place_bet(50); // Test for the time being
                self.player_action = PlayerActions::None;
            }
            PlayerActions::Check => {
                println!("Player checked");
                self.player_action = PlayerActions::None;
            }
            PlayerActions::Call => {
                println!("Player called");
                self.player_action = PlayerActions::None;
            }
            PlayerActions::Fold => {
                println!("Player has folded");
                self.winner_index = Some(1); // For the time being the other play wins, this is just for testing purpose
                self.game_state = GameState::Showdown;
                self.player_action = PlayerActions::None;
            }
            PlayerActions::None => {},
        }

        if all_acted {
            self.elapsed_time += delta;

            // Handle game-phases 
            match self.game_state {
                GameState::Preflop => {
                    if self.elapsed_time > 1.0 {
                        // Deal flop (3 cards)
                        if let Ok(flop_cards) = self.backend_game.deck.draw(3) {
                            self.backend_game.board.extend(flop_cards);
                        }
                        self.reset_actions();
                        self.game_state = GameState::Flop;
                    }
                },
                GameState::Flop => {
                    if self.elapsed_time > 2.0 {
                        // Deal turn (1 card)
                        if let Ok(turn_card) = self.backend_game.deck.draw(1) {
                            self.backend_game.board.extend(turn_card);
                        }
                        self.reset_actions();
                        self.game_state = GameState::Turn;
                    }
                },
                GameState::Turn => {
                    if self.elapsed_time > 2.0 {
                        // Deal river (1 card)
                        if let Ok(river_card) = self.backend_game.deck.draw(1) {
                            self.backend_game.board.extend(river_card);
                        }
                        self.reset_actions();
                        self.game_state = GameState::River;
                    }
                },
                GameState::River => {
                    if self.elapsed_time > 2.0 {
                        self.game_state = GameState::Showdown;
                        // Determine winner using backend evaluation
                        self.winner_index = Some(self.determine_winner());
                    }
                },
                _ => {}
                }
            }
        Ok(())
    }

    fn draw(&mut self, context: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(context, Color::from_rgb(34, 139, 34)); // Background
        
        // Screen size
        let (screen_width, screen_height) = context.gfx.drawable_size();

        // Table size
        let table_width = 1000.;
        let table_height = 400.0;
        let table_x = (screen_width - table_width) / 2.0;
        let table_y = (screen_height - table_height) / 2.0;

        let corner_radius = table_height / 2.0;
        let border_thickness = 15.0;

        let wood_color = Color::from_rgb(139, 65, 20);

        // Brown tree edge - rectangle
        let wood_rectangle = graphics::Mesh::new_rectangle(
            context, 
            graphics::DrawMode::fill(), 
            Rect::new(
                table_x + corner_radius - border_thickness, 
                table_y - border_thickness, 
                table_width - 2.0 * corner_radius + 2.0 * border_thickness, 
                table_height + 2.0 * border_thickness
            ),
            wood_color,
        )?;

        canvas.draw(&wood_rectangle, graphics::DrawParam::default());

        //Two wood edges on the on each side - circles
        let left_wood = graphics::Mesh::new_circle(
            context,
            graphics::DrawMode::fill(),
            Vec2::new(table_x + corner_radius, table_y + table_height / 2.0), 
            corner_radius + border_thickness, 
            0.1, 
            wood_color,
        )?;
        canvas.draw(&left_wood, graphics::DrawParam::default());

        let right_wood = graphics::Mesh::new_circle(
            context,
            graphics::DrawMode::fill(),
            Vec2::new(table_x + table_width - corner_radius, table_y + table_height / 2.0), 
            corner_radius + border_thickness, 
            0.1, 
            wood_color,
        )?;
        canvas.draw(&right_wood, graphics::DrawParam::default());

        // Green rectangle in the middle
        let table_rectangle = graphics::Mesh::new_rectangle(
            context, 
            graphics::DrawMode::fill(), 
            Rect::new(table_x + corner_radius, table_y, table_width - 2.0 * corner_radius, table_height),
            Color::from_rgb(0, 100, 0)
        )?;

        canvas.draw(&table_rectangle, graphics::DrawParam::default());

        // Two green circles on each side
        let left_circle = graphics::Mesh::new_circle(
            context,
            graphics::DrawMode::fill(),
            Vec2::new(table_x + corner_radius, table_y + table_height / 2.0), 
            corner_radius, 
            0.1, 
            Color::from_rgb(0, 100, 0)
        )?;
        canvas.draw(&left_circle, graphics::DrawParam::default());

        let right_circle = graphics::Mesh::new_circle(
            context,
            graphics::DrawMode::fill(),
            Vec2::new(table_x + table_width - corner_radius, table_y + table_height / 2.0), 
            corner_radius, 
            0.1, 
            Color::from_rgb(0, 100, 0)
        )?;
        canvas.draw(&right_circle, graphics::DrawParam::default());

        // Draw community cards
        for (i, card) in self.backend_game.board.iter().enumerate() {
            let card_key = card_to_image_key(card);
            if let Some(card_image) = self.card_images.get(&card_key) {
                canvas.draw(card_image, graphics::DrawParam::default()
                .dest(Vec2::new(250.0 + i as f32 * 110.0, 300.0))
                .scale(Vec2::new(0.14, 0.14)),
                );
            }
        }
        
        let winner_index = if self.game_state == GameState::Showdown {
            Some(self.determine_winner())
        } else {
            None
        };
        
        for (i, player) in self.players.iter().enumerate() {
            let mut display_text = player.name.clone();
        
            if self.game_state == GameState::Showdown {
                if i < self.backend_game.players.len() {
                    let (rank, hand_type) = self.backend_game.players[i].hand.evaluate(
                        &self.backend_game.board,
                        &self.backend_game.t5,
                        &self.backend_game.t7
                    );
                    display_text = format!("{}: {}", player.name, hand_type);
                }
            }
        
            let is_winner = match winner_index {
                Some(winner) => i == winner,
                None => false,
            };
        
            if is_winner {
                display_text = format!("{} wins!", display_text);
            }
        
            let name_text = graphics::Text::new(display_text);
            canvas.draw(&name_text, DrawParam::default().dest(player.position));
        
            for (j, card) in self.backend_game.players[i].hand.cards.iter().enumerate() {
                let card_key = card_to_image_key(card);
                if let Some(card_image) = self.card_images.get(&card_key) {
                    let mut parameter = DrawParam::default()
                        .dest(player.position + Vec2::new(j as f32 * 40.0, 30.0))
                        .scale(Vec2::new(0.28, 0.28));
        
                    if is_winner {
                        parameter = parameter.color(Color::from_rgb(255, 255, 150));
                    }
        
                    canvas.draw(card_image, parameter);
                }
            }
        }

        // Draw pot and chips
        canvas.draw(
            &self.chip_image,
            DrawParam::default()
            .dest(Vec2::new(200.0, 400.0))
            .scale(Vec2::new(0.3, 0.3)));
        
        let pot_text = Text::new(format!("Pot: {} chips", self.pot));
        canvas.draw(&pot_text, DrawParam::default()
            .dest(Vec2::new(200.0, 370.0))
    );

        // Draw action buttons
        let button_labbels = ["Bet", "Check", "Call", "Fold"];
        
        for (i, label) in button_labbels.iter().enumerate() {
            let x = 50.0 + i as f32 * 130.0;
            let y = 150.0;
            let rect = Rect::new(x, y, 120.0, 50.0);
            let button = graphics::Mesh::new_rectangle(
                context,
                DrawMode::fill(),
                rect,
                Color::from_rgb(50, 50, 50)
            )?;

        canvas.draw(&button, DrawParam::default());

        let text = graphics::Text::new((*label).to_string());
        canvas.draw(&text, DrawParam::default()
        .dest(Vec2::new(x + 30.0, y + 15.0))
        .scale(Vec2::new(1.5, 1.5)),
        );
        }
        canvas.finish(context)?;
        Ok(())
    }

    fn mouse_button_down_event(
            &mut self,
            _context: &mut Context,
            button: MouseButton,
            x: f32,
            y: f32,
        ) -> GameResult {
            if button == MouseButton::Left {
                let buttons = [
                    (PlayerActions::Bet, Rect::new(50.0, 150.0, 120.0, 50.0)),
                    (PlayerActions::Check, Rect::new(180.0, 150.0, 120.0, 50.0)),
                    (PlayerActions::Call, Rect::new(310.0, 150.0, 120.0, 50.0)),
                    (PlayerActions::Fold, Rect::new(440.0, 150.0, 120.0, 50.0)),
                ];

                for (action, rect) in buttons.iter() {
                    if rect.contains([x, y]).into() {
                        println!("Player chose to {:?}", action);
                        self.player_action = *action;

                        if self.current_player_index < self.player_actions_done.len() {
                            self.player_actions_done[self.current_player_index] = true;
                            self.current_player_index = (self.current_player_index + 1) % self.players.len();
                        }
                    }
                }
            }
        Ok(())
    }
    fn key_down_event(
            &mut self,
            _context: &mut Context,
            input: KeyInput,
            _repeated: bool,
        ) -> GameResult {
        if let Some(KeyCode::R) = input.keycode {
            self.reset_game();
        }
        Ok(())
    }
}