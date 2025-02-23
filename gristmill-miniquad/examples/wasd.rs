use std::path::Path;

use gristmill::{
    color::Color,
    math::{Pos2, Rect, Vec2},
    scene2d::{Instance, ScrollCamera, UvRect},
    DrawMetrics,
};
use gristmill_miniquad::{
    Context, DrawParams, Game, InputEvent, KeyCode, Renderer2D, Sprite2D, Stage2D, Texture,
    WindowConfig, WindowSetup,
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

struct GameRenderer {
    context: Context,
    renderer: Renderer2D,
    stage: Stage2D<()>,
    camera: ScrollCamera,
}

impl GameRenderer {
    fn render(&mut self) -> DrawMetrics {
        self.stage.set_camera((), self.camera.camera());
        self.renderer
            .render(&mut self.context, &mut self.stage, Color::BLACK)
    }
}

struct MyGame {
    _assets: GameAssets,
    input: GameInput,
    renderer: GameRenderer,
    player: Sprite2D,
}

impl Game for MyGame {
    fn init(mut context: Context, screen_size: Vec2) -> Self {
        let assets = GameAssets::load(&mut context);
        let renderer = Renderer2D::new(&mut context, None);
        let player = Sprite2D {
            params: DrawParams::from_texture(&assets.player, 0),
            instance: Instance {
                rect: Rect::from_center_size(Pos2::ZERO, assets.player.size().to_vec2()),
                uv: UvRect::default(),
                color: Color::WHITE,
            },
        };
        MyGame {
            _assets: assets,
            input: GameInput::default(),
            renderer: GameRenderer {
                context,
                renderer,
                stage: Stage2D::new(),
                camera: ScrollCamera {
                    screen_size,
                    center: Pos2::ZERO,
                    scale: 1.0,
                },
            },
            player,
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
        self.player.instance.rect = self.player.instance.rect.translate(movement);
    }

    fn resize(&mut self, screen_size: Vec2) {
        self.renderer.camera.screen_size = screen_size;
    }

    fn draw(&mut self) -> DrawMetrics {
        self.player.draw(&mut self.renderer.stage.get_layer(()));
        self.renderer.render()
    }
}

fn main() {
    gristmill::asset::set_base_path("examples/assets").unwrap();
    gristmill_miniquad::start::<MyGame>(
        WindowSetup::from_title("WASD Example".to_string()),
        WindowConfig::default(),
    );
}
