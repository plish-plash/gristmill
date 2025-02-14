use std::path::Path;

use gristmill::{
    color::Color,
    math::{Pos2, Rect, Vec2},
    scene2d::{sprite::Sprite, Camera, UvRect},
    DrawMetrics,
};
use gristmill_miniquad::{
    Context, DrawParams, Game, InputEvent, KeyCode, Renderer2D, Scene2D, Texture, WindowConfig,
    WindowSetup,
};

type Layer = u32;

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

struct MyGame {
    assets: GameAssets,
    input: GameInput,
    context: Context,
    renderer: Renderer2D,
    camera: Camera,
    scene: Scene2D<Layer>,
    player_position: Pos2,
}

impl Game for MyGame {
    fn init(mut context: Context, screen_size: Vec2) -> Self {
        let assets = GameAssets::load(&mut context);
        let renderer = Renderer2D::new(&mut context, None);
        MyGame {
            assets,
            input: GameInput::default(),
            context,
            renderer,
            camera: Camera {
                screen_size,
                center: Pos2::ZERO,
                scale: 1.0,
            },
            scene: Scene2D::new(),
            player_position: Pos2::ZERO,
        }
    }

    fn input(&mut self, event: InputEvent) {
        self.input.process(event);
    }

    fn update(&mut self, dt: f32) {
        const SPEED: f32 = 128.0;
        if self.input.up {
            self.player_position.y -= SPEED * dt;
        }
        if self.input.down {
            self.player_position.y += SPEED * dt;
        }
        if self.input.left {
            self.player_position.x -= SPEED * dt;
        }
        if self.input.right {
            self.player_position.x += SPEED * dt;
        }
    }

    fn resize(&mut self, screen_size: Vec2) {
        self.camera.screen_size = screen_size;
    }

    fn draw(&mut self) -> DrawMetrics {
        Sprite {
            layer: 0,
            params: DrawParams::texture(&self.assets.player),
            rect: Rect::from_center_size(self.player_position, self.assets.player.size().to_vec2()),
            uv: UvRect::default(),
            color: Color::WHITE,
        }
        .draw(&mut self.scene);

        self.renderer.begin_render(&mut self.context, Color::BLACK);
        self.renderer
            .set_camera(&mut self.context, self.camera.transform());
        self.scene.draw(&mut self.context, &mut self.renderer, ..);
        self.renderer.end_render(&mut self.context)
    }
}

fn main() {
    gristmill::asset::set_base_path("examples/assets").unwrap();
    gristmill_miniquad::start::<MyGame>(
        WindowSetup::with_title("WASD Example".to_string()),
        WindowConfig::default(),
    );
}
