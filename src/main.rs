use ggez::glam::Vec2;
use ggez::{Context, ContextBuilder, GameResult};
use ggez::graphics::{self, Color, Rect};
use ggez::event::{self, EventHandler};

fn main() {
    // Make a Context.
    let (mut context, event_loop) = ContextBuilder::new("Poker", "Gustav Häggström")
        .build()
        .expect("Failed to create ggez context!");

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let my_game = MyGame::new(&mut context);

    // Run!
    event::run(context, event_loop, my_game);
}

struct MyGame {
    // Your state here...
}

impl MyGame {
    pub fn new(_context: &mut Context) -> MyGame {
        // Load/create resources such as images here.
        MyGame {
            // ...
        }
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, _context: &mut Context) -> GameResult {
        // Update code here...
        Ok(())
    }

    fn draw(&mut self, context: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(context, Color::from_rgb(34, 139, 34)); // Background
        
        // Screen size
        let (screen_width, screen_height) = context.gfx.drawable_size();

        // Table size
        let table_width = screen_width * 0.6;
        let table_height = screen_height * 0.3;
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

        canvas.finish(context)
    }
}