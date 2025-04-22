use std::collections::HashMap;
use ggez::input::keyboard::{KeyCode, KeyInput};
use rand::seq::SliceRandom;
use rand::Rng;

use ggez::glam::Vec2;
use ggez::input::mouse::{self};
use ggez::{Context, ContextBuilder, GameResult};
use ggez::graphics::{self, Color, DrawParam, Image, Rect, Text};
use ggez::event::{self, EventHandler, MouseButton};

fn main() {
    // Make a Context.
    let (mut context, event_loop) = ContextBuilder::new("Poker", "Gustav, Joel")
        .add_resource_path("./resources")
        .build()
        .expect("Failed to create ggez context!");

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let my_game = MyGame::new(&mut context);

    // Run!
    event::run(context, event_loop, my_game);
}

// Texas Hold em
enum GameState {
    Preflop, 
    Flop,
    Turn, 
    River,
    Showdown,
}

#[derive(Clone)]
struct Player {
    name: String,
    chips: u32,
    hand: Vec<String>,
    position: Vec2
}

struct MyGame {
    card_images: HashMap<String, Image>, // Multiple cards
    players: Vec<Player>,
    chip_image: Image,
    game_state: GameState,
    community_cards: Vec<String>,
    deck: Vec<String>,
    elapsed_time: f32,
    winner_index: Option<usize>,
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

fn generate_deck() -> Vec<String> {
    let suits = ["clubs", "spades", "diamonds", "hearts"];
    let values = ["2", "3", "4", "5", "6", "7", "8", "9", "10", "jack", "queen", "king", "ace"];

    let mut deck = vec![];

    for suit in suits.iter() {
        for value in values.iter() {
            deck.push(format!("{}_of_{}", value, suit));
        }
    }
    deck.shuffle(&mut rand::thread_rng());
    deck
}

impl MyGame {
    pub fn new(context: &mut Context) -> MyGame {
        let card_images = load_all_cards(context);
        let chip_image = Image::from_path(context, "/casino-poker-chip-png.webp")
        .expect("Chip image not found");

        let mut deck = generate_deck();

        let mut players = vec![Player {
            name: "Joel".to_string(),
            chips: 1000,
            hand: vec![],
            position: Vec2::new(100.0, 500.0)
        },
        Player {
            name: "Gustav".to_string(),
            chips: 1000,
            hand: vec![],
            position: Vec2::new(700.0, 500.0)
        }
        ];

        for player in players.iter_mut() {
            player.hand.push(deck.pop().unwrap());
            player.hand.push(deck.pop().unwrap());
        }
        // Load/create resources such as images here.
        MyGame {
            card_images,
            players,
            chip_image,
            game_state: GameState::Preflop,
            community_cards: vec![],
            deck,
            elapsed_time: 0.0,
            winner_index: None,
        }
    }
    fn reset_game(&mut self) {
        self.deck = generate_deck();
        self.community_cards.clear();
        self.elapsed_time = 0.0;
        self.winner_index = None;
        self.game_state = GameState::Preflop;
        for player in self.players.iter_mut() {
            player.hand.clear();
            player.hand.push(self.deck.pop().unwrap());
            player.hand.push(self.deck.pop().unwrap());
        }
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, context: &mut Context) -> GameResult {
        let delta = ggez::timer::delta(context).as_secs_f32();
        self.elapsed_time += delta;
        match self.game_state {
            GameState::Preflop => {
                if self.elapsed_time > 1.0 {
                    self.community_cards.extend((0..3).map(|_| self.deck.pop().unwrap()));
                    self.elapsed_time = 0.0;
                    self.game_state = GameState::Flop;
                }
            },
            GameState::Flop => {
                if self.elapsed_time > 2.0 {
                    self.community_cards.push(self.deck.pop().unwrap());
                    self.elapsed_time = 0.0;
                    self.game_state = GameState::Turn;
                }
            },
            GameState::Turn => {
                if self.elapsed_time > 2.0 {
                    self.community_cards.push(self.deck.pop().unwrap());
                    self.elapsed_time = 0.0;
                    self.game_state = GameState::River;
                }
            },
            GameState::River => {
                if self.elapsed_time > 2.0 {
                    self.elapsed_time = 0.0;
                    self.game_state = GameState::Showdown;
                    self.winner_index = Some(rand::thread_rng().gen_range(0..self.players.len()));
                }
            },
            _ => {}
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

        for (i, card_name) in self.community_cards.iter().enumerate() {
            if let Some(card) = self.card_images.get(card_name) {
                canvas.draw(card, graphics::DrawParam::default()
                .dest(Vec2::new(250.0 + i as f32 * 110.0, 300.0))
                .scale(Vec2::new(0.14, 0.14)),
                );
            }
        }
        for (i, player) in self.players.iter().enumerate() {
            let is_winner = Some(i) == self.winner_index;

            // Draw name
            let mut name_text = graphics::Text::new(player.name.clone());
            if is_winner  {
                name_text = graphics::Text::new(format!("{} wins!", player.name));
            }
            canvas.draw(&name_text, DrawParam::default().dest(player.position));

            // Draw hand
            for (j, card_name) in player.hand.iter().enumerate() {
                if let Some(card_image) = self.card_images.get(card_name) {
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


        canvas.draw(
            &self.chip_image,
            DrawParam::default()
            .dest(Vec2::new(200.0, 400.0))
            .scale(Vec2::new(0.3, 0.3)));

        canvas.finish(context)


    }
    
    fn mouse_button_down_event(
            &mut self,
            _ctx: &mut Context,
            button: MouseButton,
            x: f32,
            y: f32,
        ) -> Result<(), ggez::GameError> {
            if button == MouseButton::Left {
                println!("click at {}, {}", x, y);
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