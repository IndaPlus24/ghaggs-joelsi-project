mod client_net;
use client_net::run_client;
use tokio::sync::mpsc::{Sender, Receiver};

use std::{collections::HashMap, net, vec};

use ggez::{
    event::{self, EventHandler}, glam::Vec2, graphics::{self, Color, DrawMode, DrawParam, Image, Mesh, Rect, Text}, input::{gamepad::gilrs::ev, keyboard::{KeyCode, KeyInput}, mouse::MouseButton}, Context, ContextBuilder, GameResult
};

use poker::{
    structs::{
        card::Card, enums::{ClientMessage, GamePhase as GameState, PlayerActions, Rank, ServerMessage, Suit}, player::{self, Player as BackendPlayer}
    }, Game
};

#[tokio::main]
async fn main() { 
    // Make a Context.
    let (mut context, event_loop) = ContextBuilder::new("Poker", "Gustav, Joel")
        .add_resource_path("./resources")
        .build()
        .expect("Failed to create ggez context!");

    use tokio::sync::mpsc;
    let (to_server_tx, to_server_rx) = mpsc::channel(32);
    let (to_game_tx, to_game_rx) = mpsc::channel(32);
    
    // Launch WebSocket client in background
    tokio::spawn(async move {
        run_client(to_server_rx, to_game_tx).await;
    });

    let my_game = MyGame::new(&mut context, to_server_tx, to_game_rx);
    event::run(context, event_loop, my_game);
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
    pot: u32,
    player_actions_done: Vec<bool>,
    current_player_index: usize,
    slider_value: u32, 
    slider_max: u32,
    slider_dragging: bool,
    show_slider: bool, 
    bet_button_clicked: bool,
    last_raiser_index: Option<usize>,
    to_server_tx: Sender<ClientMessage>,
    to_game_rx: Receiver<ServerMessage>,
    player_id: usize,
    //game_over: bool, I'll maybe use this for GUI purpose
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
    pub fn new(context: &mut Context, to_server_tx: Sender<ClientMessage>, to_game_rx: Receiver<ServerMessage>) -> MyGame {
        let card_images = load_all_cards(context);
        let chip_image = Image::from_path(context, "/casino-poker-chip-png.webp")
            .expect("Chip image not found");

        let backend_game = Game::new(0, 0);
        // vec![
        //    FrontendPlayer {
        //        name: "Joel".to_string(),
        //        chips: 1000,
        //        backend_player: BackendPlayer::new(1000),
        //        position: Vec2::new(100.0, 500.0)
        //    },
        //    FrontendPlayer {
        //        name: "Gustav".to_string(),
        //        chips: 1000,
        //        backend_player: BackendPlayer::new(1000),
        //        position: Vec2::new(700.0, 500.0)
        //    },
        // ];


        // let slider_max = frontend_players.get(0).map(|p| p.chips).unwrap_or(0);

        let _ = to_server_tx.try_send(ClientMessage::Join("Player0".to_string()));

        MyGame {
            card_images,
            players: vec![],
            chip_image,
            game_state: GameState::Preflop,
            backend_game,
            elapsed_time: 0.0,
            winner_index: None,
            player_action: PlayerActions::None,
            pot: 0,
            player_actions_done: vec![false; 2],
            current_player_index: 0,
            slider_value: 0, 
            slider_max: 0,
            slider_dragging: false,
            show_slider: false,
            bet_button_clicked: false,
            last_raiser_index: None,
            to_server_tx: to_server_tx.clone(),
            to_game_rx: to_game_rx,
            player_id: 0,
            //game_over: false,
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

        self.slider_max = self.players.get(self.current_player_index).map(|p| p.chips).unwrap_or(0);
        self.slider_value = 0;
    }

    // Track players who have folded to see what player's turn is next
    fn find_next_active_player(&self, from: usize) -> usize{
        let mut index = from;
        if self.players.is_empty() || self.backend_game.players.is_empty() {
            return 0;
        }
        while self.players.get(index).map_or(true, |p| p.chips == 0)
            || self.backend_game.players.get(index).map_or(true, |p| p.is_folded)
        {
            index = (index + 1) % self.players.len();
        }
        index
    }
    // Could be a great helper function for GameStates
    /* 
    fn advance_phase(&mut self, next_phase: GameState, draw_count: usize) {
        if let Ok(new_cards) = self.backend_game.deck.draw(draw_count) {
            self.backend_game.board.extend(new_cards);
        }
        self.reset_actions();
        self.backend_game.reset_round();
        self.game_state = next_phase;
    }
    */
}


impl EventHandler for MyGame {
    fn update(&mut self, context: &mut Context) -> GameResult {
        let delta = ggez::timer::delta(context).as_secs_f32();
        self.elapsed_time += delta;

        // RECEIVE FROM SERVER
        while let Ok(msg) = self.to_game_rx.try_recv() {
            println!("{:?}", msg);
            match msg {
                ServerMessage::Welcome(name, id) => {
                    println!("Server says: {}", name);
                    self.player_id = id;
                },
                ServerMessage::GameState(state) => {
                    // Update pot
                    self.pot = state.pot;
                    self.game_state = match state.phase.as_str() {
                        "Preflop" => GameState::Preflop,
                        "Flop" => GameState::Flop,
                        "Turn" => GameState::Turn,
                        "River" => GameState::River,
                        "Showdown" => GameState::Showdown,
                        _ => GameState::Preflop,
                    };
                
                    // Update board
                    self.backend_game.board = state.board.clone();
                
                    // Sync players
                    self.players.clear();
                    for p in &state.players {
                        self.players.push(FrontendPlayer {
                            name: p.name.clone(),
                            chips: p.chips,
                            position: if p.id == 0 {
                                Vec2::new(100.0, 500.0)
                            } else {
                                Vec2::new(700.0, 500.0)
                            },
                            backend_player: {
                                let mut bp = BackendPlayer::new(p.chips);
                                bp.hand.cards = if let Some(cards) = p.hand.clone() {
                                    cards.to_vec()
                                } else {
                                    vec![]
                                };
                                bp.is_folded = p.is_folded;
                                bp
                            },
                        });
                    }
                },
                ServerMessage::Error(err) => println!("Server error: {}", err),
            }
        }
        //////////////////////////////////////////////////////////////////////////////////////////
        if self.players.is_empty() {
            return Ok(());
        }

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
                    let _ = self.to_server_tx.send(ClientMessage::Bet(bet_amount));
                    self.last_raiser_index = Some(self.current_player_index);
                }
                PlayerActions::Check => {
                    let _ = self.to_server_tx.send(ClientMessage::Check);
                }
                PlayerActions::Call => {
                    let _ = self.to_server_tx.send(ClientMessage::Call);
                }
                PlayerActions::Fold => {
                    let _ = self.to_server_tx.send(ClientMessage::Fold);
                }
                PlayerActions::None => return Ok(()),
            }

        // When an action is done:
        self.player_actions_done[self.current_player_index] = true;
        self.player_action = PlayerActions::None;

        // Advance to next player's turn
        if self.players.is_empty() {
            return Ok(());
        }
        let mut next_index = (self.current_player_index + 1) % self.players.len();
        while self.backend_game.players[next_index].is_folded {
            next_index = (next_index + 1) % self.players.len();
        }
        self.current_player_index = next_index;

        self.slider_max = self.players.get(self.current_player_index).map(|p| p.chips).unwrap_or(0);
        self.slider_value = self.slider_value.min(self.slider_max);
    }

        // Advancement now handled by server

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
                .dest(Vec2::new(250.0 + i as f32 * 110.0, 300.0))
                .scale(Vec2::new(0.14, 0.14)),
                );
            }
        }
        
        // Highlight when a player wins
        let winner_index = if self.game_state == GameState::Showdown {
            self.winner_index
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
        
            // Make the cards yellow to represent the winner even more
            if let Some(backend_player) = self.backend_game.players.get(i) {
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

        // Slider messurments
        let slider_x = 300.0;
        let slider_y = 100.0;
        let slider_width = 300.0;
        let knob_radius = 10.0;

        // Make boundary depending on how many chips a player has
        self.slider_max = self.players.get(self.current_player_index).map(|p| p.chips).unwrap_or(0);
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
                    (PlayerActions::Bet, Rect::new(50.0, 150.0, 120.0, 50.0)),
                    (PlayerActions::Check, Rect::new(180.0, 150.0, 120.0, 50.0)),
                    (PlayerActions::Call, Rect::new(310.0, 150.0, 120.0, 50.0)),
                    (PlayerActions::Fold, Rect::new(440.0, 150.0, 120.0, 50.0)),
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
                    let slider_y = 100.0;
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
            // self.reset_game();
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