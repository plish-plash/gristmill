use std::path::Path;

use gristmill::{
    color::Color,
    math::{Pos2, Vec2},
    scene2d::ScrollCamera,
    DrawMetrics,
};
use gristmill_miniquad::{
    Context, InputEvent, KeyCode, Renderer2D, Sprite2D, Texture, WindowConfig, WindowSetup,
};

struct GameAssets {
    player: Texture,
}

impl GameAssets {
    fn load(context: &mut Context) -> Self {
        GameAssets {
            player: Texture::load(context, Path::new("player.png")).unwrap(),
        }
    }
}

#[derive(Default)]
struct GameInput {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl GameInput {
    fn process(&mut self, event: InputEvent) {
        if let InputEvent::Key { key, pressed } = event {
            match key {
                KeyCode::Up | KeyCode::W => self.up = pressed,
                KeyCode::Down | KeyCode::S => self.down = pressed,
                KeyCode::Left | KeyCode::A => self.left = pressed,
                KeyCode::Right | KeyCode::D => self.right = pressed,
                _ => (),
            }
        }
    }
}

struct Game {
    renderer: Renderer2D,
    camera: ScrollCamera,
    input: GameInput,
    player: Sprite2D,
    _assets: GameAssets,
}

impl gristmill_miniquad::Game for Game {
    fn init(mut context: Context, screen_size: Vec2) -> Self {
        let assets = GameAssets::load(&mut context);
        let player = assets.player.sprite(Pos2::ZERO, Color::WHITE);
        Game {
            renderer: Renderer2D::new(context),
            camera: ScrollCamera {
                screen_size,
                center: Pos2::ZERO,
                scale: 1.0,
            },
            input: GameInput::default(),
            player,
            _assets: assets,
        }
    }

    fn input(&mut self, event: InputEvent) {
        self.input.process(event);
    }

    fn update(&mut self, dt: f32) {
        const SPEED: f32 = 128.0;
        let mut movement = Vec2::ZERO;
        if self.input.up {
            movement.y -= SPEED * dt;
        }
        if self.input.down {
            movement.y += SPEED * dt;
        }
        if self.input.left {
            movement.x -= SPEED * dt;
        }
        if self.input.right {
            movement.x += SPEED * dt;
        }
        self.player.translate(movement);
    }

    fn resize(&mut self, screen_size: Vec2) {
        self.camera.screen_size = screen_size;
    }

    fn draw(&mut self) -> DrawMetrics {
        self.renderer.begin_render(Color::BLACK);
        let mut batcher = self.renderer.bind_pipeline();
        batcher.set_camera(self.camera.transform());
        self.player.draw(&mut batcher);
        std::mem::drop(batcher);
        self.renderer.end_render()
    }
}

fn main() {
    gristmill::asset::set_base_path("examples/assets").unwrap();
    gristmill_miniquad::start::<Game>(
        WindowSetup::from_title("WASD Example".to_string()),
        WindowConfig::default(),
    );
}
