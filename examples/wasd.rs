use gristmill::{
    game_info,
    keyboard::{KeyCode, PhysicalKey},
    render::{wgpu, Context, SurfaceSize, Texture, TextureConfig, Uv},
    world2d::{Camera2D, Point, Quad, Rect, Renderer2D, Vector},
    Game, GameError, Image, InputEvent, KeyboardEvent, Rgba,
};

#[derive(Default)]
struct WasdInput {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl WasdInput {
    fn input_event(&mut self, event: &InputEvent) {
        if let InputEvent::Keyboard(KeyboardEvent(event)) = event {
            if let PhysicalKey::Code(key_code) = event.physical_key {
                match key_code {
                    KeyCode::KeyW | KeyCode::ArrowUp => self.up = event.state.is_pressed(),
                    KeyCode::KeyS | KeyCode::ArrowDown => self.down = event.state.is_pressed(),
                    KeyCode::KeyA | KeyCode::ArrowLeft => self.left = event.state.is_pressed(),
                    KeyCode::KeyD | KeyCode::ArrowRight => self.right = event.state.is_pressed(),
                    _ => (),
                }
            }
        }
    }
    fn movement(&self) -> Vector {
        let mut movement = Vector::zero();
        if self.up {
            movement.y -= 1.0;
        }
        if self.down {
            movement.y += 1.0;
        }
        if self.left {
            movement.x -= 1.0;
        }
        if self.right {
            movement.x += 1.0;
        }
        movement.try_normalize().unwrap_or_default()
    }
}

struct WasdGame {
    input: WasdInput,
    renderer: Renderer2D,
    player_point: Point,
    player_texture: Texture,
}

impl Game for WasdGame {
    fn load(context: &Context, surface_format: wgpu::TextureFormat) -> Result<Self, GameError> {
        let texture_config = TextureConfig::new(context, wgpu::FilterMode::Linear);
        let renderer = Renderer2D::new(context, surface_format, &texture_config);
        let player_texture = Image::load_texture(context, &texture_config, "player.png")?;
        Ok(WasdGame {
            input: WasdInput::default(),
            renderer,
            player_point: Point::zero(),
            player_texture,
        })
    }
    fn resize(&mut self, _context: &Context, size: SurfaceSize) {
        self.renderer.surface_resize(size);
    }
    fn input_event(&mut self, event: InputEvent) {
        self.input.input_event(&event);
    }
    fn update(&mut self, dt: f32) {
        const PLAYER_SPEED: f32 = 200.0;
        self.player_point += self.input.movement() * PLAYER_SPEED * dt;
    }
    fn clear_color(&self) -> Rgba {
        Rgba::BLACK
    }
    fn render(&mut self, context: &Context, pass: &mut wgpu::RenderPass) {
        self.renderer
            .render(context, pass, Camera2D::default(), |renderer| {
                let size = self.player_texture.size().cast().cast_unit();
                let mut rect = Rect::from_origin_and_size(self.player_point, size);
                rect = rect.translate(-size.to_vector() / 2.0);
                renderer.set_texture(&mut self.player_texture);
                renderer.draw(Quad {
                    transform: Quad::rect_transform(rect),
                    uv: Uv::FULL,
                    color: Rgba::WHITE,
                });
            });
    }
}

fn main() {
    gristmill::run_game::<WasdGame>(game_info!("WASD Example"));
}
