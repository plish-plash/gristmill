use std::{collections::HashMap, hash::Hash, path::Path};

use emath::{Align2, Pos2, Rect, Vec2};
use serde::Deserialize;

use super::{Instance, UvRect};
use crate::{
    asset::{Asset, AssetError, YamlAsset},
    color::Color,
    Batcher, Pipeline, Size,
};

pub struct ColorRect(pub Color, pub Rect);

impl ColorRect {
    pub fn draw<P: Pipeline<Instance = Instance>>(
        &self,
        batcher: &mut Batcher<P>,
        material: &P::Material,
    ) {
        batcher.draw(
            material,
            Instance {
                rect: self.1,
                uv: UvRect::default(),
                color: self.0,
            },
        )
    }
}

#[derive(Clone)]
pub struct Sprite<Material> {
    pub material: Material,
    pub instance: Instance,
}

impl<Material: Eq + Hash + Clone> Sprite<Material> {
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
    pub fn draw<P: Pipeline<Material = Material, Instance = Instance>>(
        &self,
        batcher: &mut Batcher<P>,
    ) {
        batcher.draw(&self.material, self.instance.clone())
    }
}

#[derive(Clone)]
pub struct NinePatchSprite<Material> {
    pub material: Material,
    pub texture_size: Size,
    pub texture_center: Rect,
    pub rect: Rect,
    pub color: Color,
}

impl<Material> NinePatchSprite<Material> {
    fn instance(&self, pos_min: Pos2, pos_max: Pos2, tex_min: Pos2, tex_max: Pos2) -> Instance {
        Instance {
            rect: Rect::from_min_max(pos_min, pos_max),
            uv: UvRect::from_region(Rect::from_min_max(tex_min, tex_max), self.texture_size),
            color: self.color,
        }
    }
}
impl<Material: Eq + Hash + Clone> NinePatchSprite<Material> {
    pub fn draw<P: Pipeline<Material = Material, Instance = Instance>>(
        &self,
        batcher: &mut Batcher<P>,
    ) {
        let texture_size = self.texture_size.to_vec2();
        let rect_center = Rect::from_min_max(
            self.rect.min + self.texture_center.min.to_vec2(),
            self.rect.max - (texture_size - self.texture_center.max.to_vec2()),
        );
        batcher.draw_all(
            &self.material,
            [
                self.instance(
                    self.rect.left_top(),
                    rect_center.left_top(),
                    Pos2::ZERO,
                    self.texture_center.left_top(),
                ),
                self.instance(
                    Pos2::new(rect_center.left(), self.rect.top()),
                    rect_center.right_top(),
                    Pos2::new(self.texture_center.left(), 0.0),
                    self.texture_center.right_top(),
                ),
                self.instance(
                    Pos2::new(rect_center.right(), self.rect.top()),
                    Pos2::new(self.rect.right(), rect_center.top()),
                    Pos2::new(self.texture_center.right(), 0.0),
                    Pos2::new(texture_size.x, self.texture_center.top()),
                ),
                self.instance(
                    Pos2::new(self.rect.left(), rect_center.top()),
                    rect_center.left_bottom(),
                    Pos2::new(0.0, self.texture_center.top()),
                    self.texture_center.left_bottom(),
                ),
                self.instance(
                    rect_center.left_top(),
                    rect_center.right_bottom(),
                    self.texture_center.left_top(),
                    self.texture_center.right_bottom(),
                ),
                self.instance(
                    rect_center.right_top(),
                    Pos2::new(self.rect.right(), rect_center.bottom()),
                    self.texture_center.right_top(),
                    Pos2::new(texture_size.x, self.texture_center.bottom()),
                ),
                self.instance(
                    Pos2::new(self.rect.left(), rect_center.bottom()),
                    Pos2::new(rect_center.left(), self.rect.bottom()),
                    Pos2::new(0.0, self.texture_center.bottom()),
                    Pos2::new(self.texture_center.left(), texture_size.y),
                ),
                self.instance(
                    rect_center.left_bottom(),
                    Pos2::new(rect_center.right(), self.rect.bottom()),
                    self.texture_center.left_bottom(),
                    Pos2::new(self.texture_center.right(), texture_size.y),
                ),
                self.instance(
                    rect_center.right_bottom(),
                    self.rect.right_bottom(),
                    self.texture_center.right_bottom(),
                    texture_size.to_pos2(),
                ),
            ],
        );
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
pub struct SpriteSheet<Material> {
    sprite: Sprite<Material>,
    texture_size: Size,
    frame_size: Vec2,
    frames: HashMap<String, Vec<Pos2>>,
    current_frame: Rect,
    current_animation: String,
    current_animation_frame: usize,
    frame_duration: f32,
    frame_time: f32,
}

impl<Material> SpriteSheet<Material> {
    pub fn new(material: Material, texture_size: Size, definition: SpriteSheetDefinition) -> Self {
        let current_animation = definition
            .frames
            .keys()
            .next()
            .expect("SpriteSheetDefinition has no animations")
            .to_string();
        let mut sprite_sheet = SpriteSheet {
            sprite: Sprite {
                material,
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
    pub fn load(material: Material, texture_size: Size, path: &Path) -> Result<Self, AssetError> {
        Ok(Self::new(
            material,
            texture_size,
            SpriteSheetDefinition::load(path)?,
        ))
    }

    pub fn sprite(&self) -> &Sprite<Material> {
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
