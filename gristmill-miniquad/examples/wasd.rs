use std::{path::Path, time::Duration};

use emath::{Align2, Pos2, Rect};
use gristmill::{
    asset::Asset,
    color::Color,
    gui::GuiInputFrame,
    render2d::{Camera, Texture},
    sprite::Sprite,
    text::FontAsset,
};
use gristmill_miniquad::{GameRenderer, InputEvent, KeyCode, WindowConfig};

struct GameAssets {
    player: Texture,
}

impl GameAssets {
    fn new() -> Self {
        GameAssets {
            player: Texture::load(Path::new("examples/assets/player.png")).unwrap(),
        }
    }
}

impl gristmill_miniquad::GameAssets for GameAssets {
    type GameState = GameState;
    fn window_config(&self) -> WindowConfig {
        WindowConfig::default()
    }
    fn fonts(&self) -> Vec<FontAsset> {
        Vec::new()
    }
}

#[derive(Default)]
struct GameInput {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl gristmill_miniquad::GameInput for GameInput {
    fn event(&mut self, event: InputEvent) {
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

struct GameState {
    assets: GameAssets,
    player_position: Pos2,
}

impl gristmill_miniquad::GameState for GameState {
    type Assets = GameAssets;
    type Input = GameInput;

    fn new(assets: GameAssets) -> Self {
        GameState {
            assets,
            player_position: Pos2::ZERO,
        }
    }

    fn update(&mut self, input: &mut GameInput, frame_time: Duration) {
        const SPEED: f32 = 128.0;
        let dt = frame_time.as_secs_f32();
        if input.up {
            self.player_position.y -= SPEED * dt;
        }
        if input.down {
            self.player_position.y += SPEED * dt;
        }
        if input.left {
            self.player_position.x -= SPEED * dt;
        }
        if input.right {
            self.player_position.x += SPEED * dt;
        }
    }

    fn camera(&self) -> Camera {
        Camera {
            origin: Pos2::ZERO,
            anchor: Align2::CENTER_CENTER,
            scale: 1.0,
        }
    }

    fn draw(&mut self, renderer: &mut GameRenderer, _viewport: Rect, _gui_input: GuiInputFrame) {
        renderer.quads.queue(&Sprite {
            position: self.player_position,
            align: Align2::CENTER_CENTER,
            texture: self.assets.player.clone(),
            texture_region: None,
            color: Color::WHITE,
        });
    }
}

fn main() {
    gristmill_miniquad::start(GameAssets::new);
}
