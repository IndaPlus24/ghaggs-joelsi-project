mod client_net;
use client_net::run_client;
use tokio::sync::mpsc::{Sender, Receiver};

use std::{collections::HashMap, vec};

use ggez::{
    event::{self, EventHandler}, glam::Vec2, graphics::{self, Color, DrawMode, DrawParam, Image, Rect, Text}, input::{keyboard::{KeyCode, KeyInput}, mouse::MouseButton}, Context, ContextBuilder, GameResult
};

use poker::{
    structs::{
        hand::Hand, card::Card, enums::{ClientMessage, GamePhase as GameState, PlayerActions, Rank, ServerMessage, Suit}, player::Player as BackendPlayer
    }, Game
};

#[tokio::main]
async fn main() { 
    let (mut context, event_loop) = ContextBuilder::new("Poker", "Gustav, Joel")
        .add_resource_path("./resources")
        .build()
        .expect("Failed to create ggez context!");

    use tokio::sync::mpsc;
    let (to_server_tx, to_server_rx) = mpsc::channel(32);
    let (to_game_tx, to_game_rx) = mpsc::channel(32);
    
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
    local_players_turn: bool,
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
    if let Ok(image) = Image::from_path(context, "/PNG-cards-1.3/card-backside.png") {
        cards.insert("card-backside".to_string(), image);
    }
    cards
}

impl MyGame {
    pub fn new(context: &mut Context, to_server_tx: Sender<ClientMessage>, to_game_rx: Receiver<ServerMessage>) -> MyGame {
        let card_images = load_all_cards(context);
        let chip_image = Image::from_path(context, "/casino-poker-chip-png.webp")
            .expect("Chip image not found");

        let backend_game = Game::new(0, 0);

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
            local_players_turn: false,
            //game_over: false,
        }
    }
}


impl EventHandler for MyGame {
    fn update(&mut self, context: &mut Context) -> GameResult {
        let delta = ggez::timer::delta(context).as_secs_f32();
        self.elapsed_time += delta;

        // RECEIVE FROM SERVER
        while let Ok(msg) = self.to_game_rx.try_recv() {
            // DEBUG
            // println!("{:?}", msg);
            match msg {
                ServerMessage::Welcome(name, id) => {
                    self.player_id = id;
                }
                ServerMessage::GameState(state) => {
                    self.game_state = state.phase;
                    self.pot = state.pot;
                    self.winner_index = state.winner;
                    self.backend_game.board = state.board.clone();
                    self.players.clear();
                    let positions: Vec<Vec2> = vec![
                        Vec2::new(100.0, 500.0),
                        Vec2::new(500.0, 500.0),
                        Vec2::new(500.0, 200.0),
                        Vec2::new(100.0, 500.0),
                    ];

                    for (i, player) in state.players.iter().enumerate() {
                        let hole_cards = player.hand.clone().map(|arr| arr.to_vec()).unwrap_or_default();
                        self.players.push(FrontendPlayer {
                            name: player.name.clone(),
                            chips: player.chips.chips,
                            backend_player: BackendPlayer {
                                hand: Hand { cards: hole_cards },
                                chips: player.chips.clone(),
                                is_folded: player.is_folded,
                                name: "temp".to_string(),
                            },
                            position: positions[i],
                        });
                    }
                    self.current_player_index = state.current_turn;
                    self.local_players_turn = self.current_player_index == self.player_id;
                    self.player_action = PlayerActions::None;
                }
                ServerMessage::Error(err) => {
                    eprintln!("Server error: {}", err);
                    self.local_players_turn = true;
                }
                _ => {}
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
        if self.local_players_turn {
            match self.player_action {
                PlayerActions::Bet => {
                    let bet_amount = self.slider_value;
                    let _ = self.to_server_tx.try_send(ClientMessage::Bet(bet_amount));
                    self.last_raiser_index = Some(self.current_player_index);
                    self.local_players_turn = false;
                }
                PlayerActions::Check => {
                    let _ = self.to_server_tx.try_send(ClientMessage::Check);
                    self.local_players_turn = false;
                }
                PlayerActions::Call => {
                    let _ = self.to_server_tx.try_send(ClientMessage::Call);
                    self.local_players_turn = false;
                }
                PlayerActions::Fold => {
                    let _ = self.to_server_tx.try_send(ClientMessage::Fold);
                    self.local_players_turn = false;
                }
                PlayerActions::None => return Ok(()),
            }


            // When an action is done:
            self.player_action = PlayerActions::None;

            // Advance to next player's turn
            //if self.players.is_empty() {
            //    return Ok(());
            //}
            //let mut next_index = (self.current_player_index + 1) % self.players.len();
            //while self.players[next_index].backend_player.is_folded {
            //    next_index = (next_index + 1) % self.players.len();
            //}
            //self.current_player_index = next_index;

            self.slider_max = self.players.get(self.current_player_index).map(|p| p.chips).unwrap_or(0);
            self.slider_value = self.slider_value.min(self.slider_max);
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
                .dest(Vec2::new(250.0 + i as f32 * 110.0, 220.0))
                .scale(Vec2::new(0.14, 0.14)),
                );
            }
        }
        
        // Highlight when a player wins
        let winner = if self.game_state == GameState::Showdown {
            self.winner_index
        } else {
            None
        };
        
        for (i, player) in self.players.iter().enumerate() {
            let mut display_text = if let Some(w) = winner {
                if i == w {
                    format!("{} wins!", player.name)
                } else {
                    format!("{}: {}",player.name.clone(), player.chips)
                }
            } else {
                format!("{}: {}",player.name.clone(), player.chips)
            };

            if self.current_player_index == i {
                display_text = format!("*{}", display_text);
            }
        
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
        
            let winner = if self.game_state == GameState::Showdown {
                self.winner_index
            } else {
                None
            };

            let name_text = graphics::Text::new(display_text);
            canvas.draw(&name_text, DrawParam::default().dest(player.position));
        
            // Make the cards yellow to represent the winner even more
            for (i, player) in self.players.iter().enumerate() {
                for (j, card) in player.backend_player.hand.cards.iter().enumerate() {

                    let card_key: String = if self.game_state == GameState::Showdown || i == self.player_id { card_to_image_key(card) } else { "card-backside".to_string() };
                    if let Some(card_image) = self.card_images.get(&card_key) {
                        let mut parameter = DrawParam::default()
                            .dest(player.position + Vec2::new(j as f32 * 40.0, 30.0))
                            .scale(Vec2::new(0.28, 0.28));
                
                        if winner == Some(i) {
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
            .scale(Vec2::new(0.1, 0.1)));
        
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