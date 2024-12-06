use std::{collections::HashMap, path::Path, time::Duration};

use emath::{Align2, Pos2, Rect, RectTransform, Vec2};
use serde::Deserialize;

use crate::{
    asset::{Asset, AssetError, YamlAsset},
    color::Color,
    render2d::{Quad, Texture, ToQuad, UNIT_RECT},
};

#[derive(Clone)]
pub struct Sprite {
    pub position: Pos2,
    pub align: Align2,
    pub texture: Texture,
    pub texture_region: Option<Rect>,
    pub color: Color,
}

impl ToQuad for Sprite {
    fn to_quad(&self) -> (Quad, Option<Texture>) {
        let (texture_rect, size) = if let Some(region) = self.texture_region {
            let transform = RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, self.texture.size),
                UNIT_RECT,
            );
            (transform.transform_rect(region), region.size())
        } else {
            (UNIT_RECT, self.texture.size)
        };
        let rect = self.align.anchor_size(self.position, size);
        (
            Quad {
                rect,
                texture_rect,
                color: self.color,
            },
            Some(self.texture.clone()),
        )
    }
}

pub struct ColorRect(pub Color, pub Rect);

impl ToQuad for ColorRect {
    fn to_quad(&self) -> (Quad, Option<Texture>) {
        (
            Quad {
                rect: self.1,
                texture_rect: UNIT_RECT,
                color: self.0,
            },
            None,
        )
    }
}

#[derive(Deserialize)]
struct SpriteSheetDefinition {
    fps: f32,
    frame_size: Vec2,
    frames: HashMap<String, Vec<Pos2>>,
}

impl Default for SpriteSheetDefinition {
    fn default() -> Self {
        Self {
            fps: 24.,
            frame_size: Vec2::ZERO,
            frames: Default::default(),
        }
    }
}

impl YamlAsset for SpriteSheetDefinition {}

#[derive(Clone)]
pub struct SpriteSheet {
    sprite: Sprite,
    frame_size: Vec2,
    frames: HashMap<String, Vec<Pos2>>,
    current_frame: Rect,
    current_animation: String,
    current_animation_frame: usize,
    frame_duration: f32,
    frame_time: f32,
}

impl SpriteSheet {
    pub fn sprite(&self) -> &Sprite {
        &self.sprite
    }
    pub fn sprite_mut(&mut self) -> &mut Sprite {
        &mut self.sprite
    }
    pub fn frame_size(&self) -> Vec2 {
        self.frame_size
    }
    pub fn set_animation(&mut self, animation: &str) {
        self.current_animation = animation.to_string();
        self.frame_time = 0.0;
        self.set_animation_frame(0);
    }
    pub fn set_animation_frame(&mut self, frame: usize) {
        if let Some(frames) = self.frames.get(&self.current_animation) {
            self.current_animation_frame = frame % frames.len();
            self.current_frame =
                Rect::from_min_size(frames[self.current_animation_frame], self.frame_size);
            self.sprite.texture_region = Some(self.current_frame);
        } else {
            if self.current_animation.is_empty() {
                log::error!("Animation not set");
            } else {
                log::error!("No animation called {}", self.current_animation);
            }
        }
    }
    pub fn animate(&mut self, frame_time: Duration) {
        self.frame_time += frame_time.as_secs_f32();
        if self.frame_time >= self.frame_duration {
            self.frame_time -= self.frame_duration;
            self.set_animation_frame(self.current_animation_frame + 1);
        }
    }
}

impl Asset for SpriteSheet {
    fn load(path: &Path) -> Result<Self, AssetError> {
        let frames = SpriteSheetDefinition::load(path)?;
        let texture = Texture::load(&path.with_extension("png"))?;
        Ok(SpriteSheet {
            sprite: Sprite {
                position: Pos2::ZERO,
                align: Align2::CENTER_CENTER,
                texture,
                texture_region: None,
                color: Color::WHITE,
            },
            frame_size: frames.frame_size,
            frames: frames.frames,
            current_frame: Rect::ZERO,
            current_animation: String::new(),
            current_animation_frame: 0,
            frame_duration: 1. / frames.fps,
            frame_time: 0.,
        })
    }
}
