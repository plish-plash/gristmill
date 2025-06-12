use gristmill::{
    game_info,
    render::{wgpu, Context, SurfaceSize},
    Game, GameError, InputEvent, Rgba,
};

struct ErrorGame;

impl Game for ErrorGame {
    fn load(_context: &Context, _surface_format: wgpu::TextureFormat) -> Result<Self, GameError> {
        Err(GameError::from_string(
            "An error occurred while loading the game.".to_string(),
        ))
    }
    fn resize(&mut self, _context: &Context, _size: SurfaceSize) {}
    fn input_event(&mut self, _event: InputEvent) {}
    fn update(&mut self, _dt: f32) {}
    fn clear_color(&self) -> Rgba {
        Rgba::BLACK
    }
    fn render(&mut self, _context: &Context, _pass: &mut wgpu::RenderPass) {}
}

fn main() {
    gristmill::run_game::<ErrorGame>(game_info!("Error Example"));
}
