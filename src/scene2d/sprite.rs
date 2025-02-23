use std::{collections::HashMap, path::Path};

use emath::{Align2, Pos2, Rect, Vec2};
use serde::Deserialize;

use super::{Instance, UvRect};
use crate::{
    asset::{Asset, AssetError, YamlAsset},
    color::Color,
    Batcher, Size,
};

#[derive(Clone)]
pub struct Sprite<T> {
    pub params: T,
    pub instance: Instance,
}

impl<T: Eq + PartialOrd + Clone> Sprite<T> {
    pub fn translate(&mut self, translate: Vec2) {
        self.instance.rect = self.instance.rect.translate(translate);
    }
    pub fn scale(&mut self, scale: f32) {
        self.instance.rect = self.instance.rect.scale_from_center(scale);
    }
    pub fn with_position(&self, position: Pos2, align: Align2) -> Self {
        let mut sprite = self.clone();
        sprite.instance.rect = align.anchor_size(position, self.instance.rect.size());
        sprite
    }
    pub fn draw(&self, batcher: &mut Batcher<T, Instance>) {
        batcher.add(self.params.clone(), self.instance.clone())
    }
}

pub struct ColorRect(pub Color, pub Rect);

impl ColorRect {
    pub fn draw<T: Eq + PartialOrd>(&self, batcher: &mut Batcher<T, Instance>, params: T) {
        batcher.add(
            params,
            Instance {
                rect: self.1,
                uv: UvRect::default(),
                color: self.0,
            },
        )
    }
}

#[derive(Deserialize)]
pub struct SpriteSheetDefinition {
    fps: f32,
    frame_size: Vec2,
    frames: HashMap<String, Vec<Pos2>>,
}

impl Default for SpriteSheetDefinition {
    fn default() -> Self {
        Self {
            fps: 24.0,
            frame_size: Vec2::ZERO,
            frames: Default::default(),
        }
    }
}

impl YamlAsset for SpriteSheetDefinition {}

#[derive(Clone)]
pub struct SpriteSheet<T> {
    sprite: Sprite<T>,
    texture_size: Size,
    frame_size: Vec2,
    frames: HashMap<String, Vec<Pos2>>,
    current_frame: Rect,
    current_animation: String,
    current_animation_frame: usize,
    frame_duration: f32,
    frame_time: f32,
}

impl<T> SpriteSheet<T> {
    pub fn new(params: T, texture_size: Size, definition: SpriteSheetDefinition) -> Self {
        let current_animation = definition
            .frames
            .keys()
            .next()
            .expect("SpriteSheetDefinition has no animations")
            .to_string();
        let mut sprite_sheet = SpriteSheet {
            sprite: Sprite {
                params,
                instance: Instance {
                    rect: Rect::from_min_size(Pos2::ZERO, definition.frame_size),
                    uv: UvRect::default(),
                    color: Color::WHITE,
                },
            },
            texture_size,
            frame_size: definition.frame_size,
            frames: definition.frames,
            current_frame: Rect::ZERO,
            current_animation,
            current_animation_frame: 0,
            frame_duration: 1.0 / definition.fps,
            frame_time: 0.0,
        };
        sprite_sheet.set_animation_frame(0);
        sprite_sheet
    }
    pub fn load(params: T, texture_size: Size, path: &Path) -> Result<Self, AssetError> {
        Ok(Self::new(
            params,
            texture_size,
            SpriteSheetDefinition::load(path)?,
        ))
    }

    pub fn sprite(&self) -> &Sprite<T> {
        &self.sprite
    }
    pub fn set_position(&mut self, pos: Pos2, anchor: Align2) {
        self.sprite.instance.rect = anchor.anchor_size(pos, self.frame_size);
    }
    pub fn set_color(&mut self, color: Color) {
        self.sprite.instance.color = color;
    }

    pub fn frame_size(&self) -> Vec2 {
        self.frame_size
    }
    pub fn set_animation(&mut self, animation: &str) {
        if !self.frames.contains_key(animation) {
            log::error!("SpriteSheet has no animation called '{}'", animation);
            return;
        }
        self.current_animation = animation.to_string();
        self.frame_time = 0.0;
        self.set_animation_frame(0);
    }
    pub fn set_animation_frame(&mut self, frame: usize) {
        let frames = &self.frames[&self.current_animation];
        assert!(
            !frames.is_empty(),
            "SpriteSheet animation '{}' has zero frames",
            self.current_animation
        );
        self.current_animation_frame = frame % frames.len();
        self.current_frame =
            Rect::from_min_size(frames[self.current_animation_frame], self.frame_size);
        self.sprite.instance.uv = UvRect::from_region(self.current_frame, self.texture_size);
    }
    pub fn update(&mut self, dt: f32) {
        self.frame_time += dt;
        if self.frame_time >= self.frame_duration {
            self.frame_time -= self.frame_duration;
            self.set_animation_frame(self.current_animation_frame + 1);
        }
    }
}
