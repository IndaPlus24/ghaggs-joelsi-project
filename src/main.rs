use std::{collections::HashMap, net, vec};

use ggez::{
    event::{self, EventHandler}, glam::Vec2, graphics::{self, Color, DrawMode, DrawParam, Image, Mesh, Rect, Text, TextFragment}, input::{gamepad::gilrs::ev, keyboard::{KeyCode, KeyInput}, mouse::MouseButton}, Context, ContextBuilder, GameResult
};

use ghaggs_joelsi_project::{
    structs::{
        card::Card, enums::{Rank, Suit}, player::{self, Player as BackendPlayer}
    }, Game
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
    position: Vec2,
    last_action: Option<PlayerActions>,
}
struct MyGame {
    card_images: HashMap<String, Image>, // Multiple cards
    players: Vec<FrontendPlayer>,
    chip_images: Vec<Image>, // stores pot images for different ranges
    game_state: GameState,
    backend_game: Game, // Backend game logic
    elapsed_time: f32,
    winner_index: Option<usize>,
    player_action: PlayerActions,
    pot: u32,
    player_actions_done: Vec<bool>,
    current_player_index: usize,
    slider_value: u32, 
    slider_max: u32,
    slider_dragging: bool,
    show_slider: bool, 
    bet_button_clicked: bool,
    last_raiser_index: Option<usize>,
    game_over: bool,
    game_over_message: Option<String>
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

// GUI function for loading all cards on the screen
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

impl MyGame {
    pub fn new(context: &mut Context) -> MyGame {
        let card_images = load_all_cards(context);
        let chip_images = vec![
            Image::from_path(context, "/pot1.png").unwrap(), // 0–99
            Image::from_path(context, "/pot2.png").unwrap(), // 100–499
            Image::from_path(context, "/pot3.png").unwrap(), // 500–749
            Image::from_path(context, "/pot4.png").unwrap(), // 750+
        ];

        let mut backend_game = Game::new(2, 1000); // This needs to be changed when network is integrated

        backend_game.deck.shuffle();

        // This is also gonna get changed when network is integrated
        let mut frontend_players = vec![
            FrontendPlayer {
                name: "Joel".to_string(),
                chips: 1000,
                backend_player: BackendPlayer::new(1000),
                position: Vec2::new(100.0, 500.0),
                last_action: None,
            },
            FrontendPlayer {
                name: "Gustav".to_string(),
                chips: 1000,
                backend_player: BackendPlayer::new(1000),
                position: Vec2::new(700.0, 500.0),
                last_action: None,
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

        let slider_max = frontend_players[0].chips;

        MyGame {
            card_images,
            players: frontend_players,
            chip_images,
            game_state: GameState::Preflop,
            backend_game,
            elapsed_time: 0.0,
            winner_index: None,
            player_action: PlayerActions::None,
            pot: 0,
            player_actions_done: vec![false; 2],
            current_player_index: 0,
            slider_value: 0, 
            slider_max,
            slider_dragging: false,
            show_slider: false,
            bet_button_clicked: false,
            last_raiser_index: None,
            game_over: false,
            game_over_message: None,
        }
    }

    fn sync_pot_and_chips(&mut self) {
        self.pot = self.backend_game.pot.total; // Sync pot with backend
        for (i, player) in self.players.iter_mut().enumerate() {
            player.chips = self.backend_game.players[i].chips.chips; // Sync players with chips
        }
    }
    
    // Reset the game when a player has won or pressed R (single player verison)
    fn reset_game(&mut self) {
        self.backend_game = Game::new(2, 1000);
        self.backend_game.deck.shuffle();
        for player in 0..self.backend_game.players.len() {
            if let Ok(cards) = self.backend_game.deck.draw(2) {
                self.backend_game.players[player].hand.cards = cards;
                self.last_raiser_index = None;
                self.game_over = false;
                self.game_over_message = None;
                self.sync_pot_and_chips();
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

    // Reset actions to be able to do all actions in the next game-phase
    fn reset_actions(&mut self) {
        self.player_actions_done = vec![false; self.players.len()];
        self.elapsed_time = 0.0;
        self.player_action = PlayerActions::None;

        // Reset who raised
        self.last_raiser_index = None;

        // Start with next non-folded player
        self.current_player_index = self.players
        .iter()
        .position(|predicate| !predicate.backend_player.is_folded)
        .unwrap_or(0);

        self.slider_max = self.players[self.current_player_index].chips;
        self.slider_value = 0;
    }

    // Track players who have folded to see what player's turn is next
    fn find_next_active_player(&self, from: usize) -> usize{
        let mut index = from;
        while self.players[index].chips == 0 || self.backend_game.players[index].is_folded {
            index = (index + 1) % self.players.len();
        } 
        index
    }

    fn determine_winner(&self) -> usize {
        self.backend_game.best_hand()
    }
}


impl EventHandler for MyGame {
    fn update(&mut self, context: &mut Context) -> GameResult {
        let delta = ggez::timer::delta(context).as_secs_f32();
        self.elapsed_time += delta;

        // Slider for betting
        if self.slider_dragging && self.slider_max > 0 {
            let mouse_x = ggez::input::mouse::position(context).x;
            let relative = (mouse_x - 600.0).clamp(0.0, 300.0);
            self.slider_value = ((relative / 300.0) * self.slider_max as f32) as u32;
        }

        // Handle player actions using backend logic
        if !self.player_actions_done[self.current_player_index] {
            match self.player_action {
                PlayerActions::Bet => {
                    let bet_amount = self.slider_value;
                    if self.backend_game.bet(self.current_player_index, bet_amount).is_ok() {
                        self.last_raiser_index = Some(self.current_player_index);
                    }
                    else {
                        println!("Error with betting");
                    }
                }
            PlayerActions::Check => {
                    if let Err(error) = self.backend_game.check(self.current_player_index) {
                        println!("Check error: {}", error);
                    }
                }
            PlayerActions::Call => {
                if let Err(error) = self.backend_game.call(self.current_player_index) {
                    println!("Call error: {}", error);
                }
            }
            PlayerActions::Fold => {
                self.backend_game.fold(self.current_player_index);
            }
            PlayerActions::None => return Ok(()),
        }

        // When an action is done:
        self.sync_pot_and_chips();
        self.player_actions_done[self.current_player_index] = true;
        self.player_action = PlayerActions::None;

        // Advance to next player's turn
        let mut next_index = (self.current_player_index + 1) % self.players.len();
        while self.backend_game.players[next_index].is_folded {
            next_index = (next_index + 1) % self.players.len();
        }
        self.current_player_index = next_index;

        self.slider_max = self.players[self.current_player_index].chips;
        self.slider_value = self.slider_value.min(self.slider_max);
    }

        // Check if round is ready to advance
        let all_acted = self.player_actions_done
            .iter()
            .enumerate()
            .all(|(i, acted)| self.backend_game.players[i].is_folded || *acted);

        let everyone_matched = self.backend_game.non_folded_players_match_bet();

        // Advance to the next phase. Here's also where all the gamephases are handled
        if all_acted && everyone_matched {
            self.elapsed_time += delta;
            
            match self.game_state {
                // First phase
                GameState::Preflop => {
                        // Deal flop (3 cards)
                        if let Ok(flop_cards) = self.backend_game.deck.draw(3) {
                            self.backend_game.board.extend(flop_cards);
                        }
                        self.reset_actions();
                        self.backend_game.reset_round();
                        self.game_state = GameState::Flop;
                    
                },
                // Second phase
                GameState::Flop => {
                        // Deal turn (1 card)
                        if let Ok(turn_card) = self.backend_game.deck.draw(1) {
                            self.backend_game.board.extend(turn_card);
                        }
                        self.reset_actions();
                        self.backend_game.reset_round();
                        self.players[self.current_player_index].last_action = Some(self.player_action.clone());
                        self.game_state = GameState::Turn;
                    
                },
                // Third phase
                GameState::Turn => {
                        // Deal river (1 card)
                        if let Ok(river_card) = self.backend_game.deck.draw(1) {
                            self.backend_game.board.extend(river_card);
                        }
                        self.reset_actions();
                        self.backend_game.reset_round();
                        self.players[self.current_player_index].last_action = Some(self.player_action.clone());
                        self.game_state = GameState::River;
                    
                },
                // Fourth phase
                GameState::River => {
                        self.game_state = GameState::Showdown;                    
                },
                // Fifth and last phase
                GameState::Showdown => {
                    // Set round winner once on first showdown frame
                    if self.winner_index.is_none() {
                        self.winner_index = Some(self.determine_winner());
                        self.backend_game.award_pot_to_winner();
                        self.elapsed_time = 0.0; // Reset timer when entering showdown
                    }
                    // Check how many are alive and if someone has won the game
                    if self.elapsed_time > 3.0 {
                        let alive_players: Vec<_> = self.players
                        .iter()
                        .filter(|predicate| predicate.chips > 0)
                        .collect();
                        if alive_players.len() <= 1 {
                            if let Some(winner) = alive_players.first() {
                                self.game_over_message = Some(format!("Game Over! {} wins!", winner.name));
                            } else {
                                self.game_over_message = Some("Game Over! No chips left.".to_string());
                            }
                            self.game_over = true;
                        }
                        // Otherwise restart round and keep going
                        else {
                            self.backend_game.reset_round();
                            self.sync_pot_and_chips();
                            self.reset_actions();
                            self.game_state = GameState::Preflop;
                            self.winner_index = None;
                            self.current_player_index = self.find_next_active_player(0);
                            self.slider_max = self.players[self.current_player_index].chips;
                            self.slider_value = 0;
                        }
                    }
                }
                }
            }
        Ok(())
    }

    // All GUI drawings/imports to the screen
    fn draw(&mut self, context: &mut Context) -> GameResult {
        // Background
        let mut canvas = graphics::Canvas::from_frame(context, Color::from_rgb(34, 139, 34));
        
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
                .dest(Vec2::new(200.0 + i as f32 * 110.0, 200.0))
                .scale(Vec2::new(0.14, 0.14)),
                );
            }
        }
        
        // Highlight when a player wins
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
        
            // Draw players hand (singleplayer version)
            for (j, card) in self.backend_game.players[i].hand.cards.iter().enumerate() {
                let card_key = card_to_image_key(card);
                if let Some(card_image) = self.card_images.get(&card_key) {
                    let mut parameter = DrawParam::default()
                        .dest(player.position + Vec2::new(j as f32 * 40.0, 30.0))
                        .scale(Vec2::new(0.28, 0.28));
                    // Make the cards yellow to represent the winner even more
                    if is_winner {
                        parameter = parameter.color(Color::from_rgb(255, 255, 150));
                    }
        
                    canvas.draw(card_image, parameter);
                }
            }
        }

        /*
        // Draw players hand (multiplayer version)
        for (i, player) in self.players.iter().enumerate() {
            let is_you = i == self.local_player_index; // Defined later in backend/network
            let is_showdown = self.game_state = GameState::Showdown;

            for (j, card) in player.backend_player.hand.cards.iter().enumerate() {
                let card_image = if is_you || is_showdown {
                    &self.card_images[card.index()] // the card.index() method should give each card a unique ID for image lookup
                }
                else {
                    &self.card_back_image // an image of the back of a card
                };
                let position = player.position + Vec2::new((j * 40) as f32, 0.0);
                canvas.draw(card_image, DrawParam::default().dest(position));
            }
        }
        */

        // Set pot ranges for the different pot-pngs
        let chip_index = match self.pot {
            0..= 99 => 0,
            100..= 499 => 1,
            500..= 749 => 2,
            _ => 3,
        };
        let chip_image = &self.chip_images[chip_index];
        // Draw pot
        canvas.draw(
            chip_image,
            DrawParam::default()
            .dest(Vec2::new(300.0, 300.0))
            .scale(Vec2::new(0.3, 0.3)));
        
        let pot_text = Text::new(format!("Pot: {} chips", self.pot));
        canvas.draw(&pot_text, DrawParam::default()
            .dest(Vec2::new(100.0, 370.0))
    );

        // Draw action buttons
        let button_labbels = ["Bet", "Check", "Call", "Fold"];
        
        for (i, label) in button_labbels.iter().enumerate() {
            let x = 50.0 + i as f32 * 130.0;
            let y = 100.0;
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

        // Slider messurments
        let slider_x = 300.0;
        let slider_y = 50.0;
        let slider_width = 300.0;
        let knob_radius = 10.0;

        // Make boundary depending on how many chips a player has
        self.slider_max = self.players[self.current_player_index].chips;
        if self.slider_value > self.slider_max {
            self.slider_value = self.slider_max;
            }

        // Slider shows when bet is clicked
        if self.show_slider {
            let track = graphics::Mesh::new_rectangle(
                context, 
                DrawMode::fill(), 
                Rect::new(slider_x, slider_y, slider_width, 4.0), 
                Color::WHITE
            )?;
        

        canvas.draw(&track, DrawParam::default());

        // Movable knob for the slider
        let knob_position = slider_x + (self.slider_value as f32 / self.slider_max as f32) * slider_width;
        let knob = graphics::Mesh::new_circle(
            context, 
            DrawMode::fill(), 
            Vec2::new(knob_position, slider_y + 2.0),
            knob_radius,
            0.1,
            Color::YELLOW,
        )?;

        canvas.draw(&knob, DrawParam::default());

        let value_text = Text::new(format!("Bet: {} chips", self.slider_value));
        canvas.draw(&value_text, DrawParam::default().dest(Vec2::new(slider_x, slider_y + 20.0)));
        }

        // Draw textbox with chosen playeraction 
        for player in &self.players {
            if let Some(action) = &player.last_action {
                let text = Text::new(format!("{:?}", action));
                canvas.draw(
                    &text, 
                    DrawParam::default()
                    .dest(player.position + Vec2::new(0.0, 150.0))
                    .color(Color::WHITE)
                );
            }
        }

        // Draw game over text
        if let Some(ref message) = self.game_over_message {
            let fragment = TextFragment::new(message.as_str()).scale(36.0);
            let text = Text::new(fragment);
            let screen_center = Vec2::new(screen_width / 2.0, screen_height / 2.0);
            canvas.draw(&text, DrawParam::default().dest(screen_center));
        }
        canvas.finish(context)?;
        Ok(())
    }

    // Mousehandling
    fn mouse_button_down_event(
            &mut self,
            _context: &mut Context,
            button: MouseButton,
            x: f32,
            y: f32,
        ) -> GameResult {
            if button == MouseButton::Left {
                self.slider_dragging = false;

                let buttons = [
                    (PlayerActions::Bet, Rect::new(50.0, 100.0, 120.0, 50.0)),
                    (PlayerActions::Check, Rect::new(180.0, 100.0, 120.0, 50.0)),
                    (PlayerActions::Call, Rect::new(310.0, 100.0, 120.0, 50.0)),
                    (PlayerActions::Fold, Rect::new(440.0, 100.0, 120.0, 50.0)),
                ];

                for (action, rect) in buttons.iter() {
                    if rect.contains([x, y]).into() {
                        println!("Player chose to {:?}", action);

                        if *action == PlayerActions::Bet {
                            if self.bet_button_clicked {
                                println!("Confirmed bet of {} chips", self.slider_value);
                                self.player_action = PlayerActions::Bet;
                                self.show_slider = false;
                                self.bet_button_clicked = false;
                            } else {
                                self.show_slider = true;
                                self.bet_button_clicked = true; 
                                self.player_action = PlayerActions::None;
                            }
                        } else {
                            self.player_action = *action;
                            self.show_slider = false;
                            self.bet_button_clicked = false;
                        }
                    break;
                    }
                }

                if self.show_slider {
                    let slider_x = 300.0;
                    let slider_y = 50.0;
                    let slider_width = 300.0;
                    let knob_radius = 10.0;
        
                    let knob_x = slider_x + (self.slider_value as f32 / self.slider_max as f32) * slider_width;
                    let knob_y = slider_y + 2.0;

                    let knob_hitbox = Rect::new(
                        knob_x - knob_radius * 2.0,
                        knob_y - knob_radius * 2.0,
                        knob_radius * 4.0,
                        knob_radius * 4.0
                    );

                    let track_hitbox = Rect::new(
                        slider_x,
                        slider_y - 10.0,
                        slider_width,
                        20.0,
                    );
        
                    if knob_hitbox.contains::<[f32; 2]>([x, y]) || track_hitbox.contains::<[f32; 2]>([x, y]) {
                        self.slider_dragging = true;
                        // Immediately update slider value based on click position
                        let clamped_x = x.clamp(slider_x, slider_x + slider_width);
                        let percent = (clamped_x - slider_x) / slider_width;
                        self.slider_value = ((percent * self.slider_max as f32).round()) as u32;
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

    fn mouse_motion_event(
            &mut self,
            _context: &mut Context,
            x: f32,
            _y: f32,
            _dx: f32,
            _dy: f32,
        ) -> GameResult {
        if self.slider_dragging {
            let slider_x = 300.0;
            let slider_width = 300.0;

            let clamped_x = x.clamp(slider_x, slider_x + slider_width);
            let percent = (clamped_x - slider_x) / slider_width;
            self.slider_value = ((percent * self.slider_max as f32).round()) as u32;
        }
        Ok(())
    }

    fn mouse_button_up_event(
            &mut self,
            _context: &mut Context,
            button: MouseButton,
            _x: f32,
            _y: f32,
        ) -> GameResult {
        if button == MouseButton::Left {
            self.slider_dragging = false;
        }
        Ok(())
    }
}