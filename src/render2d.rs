use std::{path::Path, sync::Arc};

use emath::{Align2, Pos2, Rect, RectTransform, Vec2};

use crate::{
    asset::{Asset, AssetError, Image},
    color::Color,
    Dispatcher, Handle, QueueBuilder,
};

pub const UNIT_RECT: Rect = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));

#[derive(Clone)]
pub struct Camera {
    pub origin: Pos2,
    pub anchor: Align2,
    pub scale: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            origin: Pos2::ZERO,
            anchor: Align2::LEFT_TOP,
            scale: 1.0,
        }
    }
}

impl Camera {
    pub fn viewport(&self, screen_size: Vec2) -> Rect {
        self.anchor
            .anchor_size(self.origin, screen_size / self.scale)
    }
    pub fn screen_transform(&self, screen_size: Vec2) -> RectTransform {
        RectTransform::from_to(
            self.viewport(screen_size),
            Rect::from_min_size(Pos2::ZERO, screen_size),
        )
    }
    pub fn render_transform(&self, screen_size: Vec2) -> RectTransform {
        RectTransform::from_to(self.viewport(screen_size), UNIT_RECT)
    }
}

#[derive(Clone)]
pub struct Texture {
    pub handle: Handle,
    pub size: Vec2,
}

impl PartialEq for Texture {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.handle, &other.handle)
    }
}
impl Eq for Texture {}

extern "Rust" {
    fn load_texture(image: Image) -> Texture;
}

impl Asset for Texture {
    fn load(path: &Path) -> Result<Self, AssetError> {
        let image = Image::load(path)?;
        unsafe { Ok(load_texture(image)) }
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct Quad {
    pub rect: Rect,
    pub texture_rect: Rect,
    pub color: Color,
}

impl Quad {
    pub fn transform_and_clip(mut self, render_transform: &RectTransform) -> Option<Self> {
        self.rect = render_transform.transform_rect(self.rect);
        if self.rect.intersects(UNIT_RECT) {
            Some(self)
        } else {
            None
        }
    }
}

pub trait ToQuad {
    fn to_quad(&self) -> (Quad, Option<Texture>);
}

pub struct QuadDrawQueue {
    dispatcher: Dispatcher,
    queue: QueueBuilder<Quad>,
    textures: Vec<Option<Texture>>,
    current_texture: Option<Option<Texture>>,
    render_transform: RectTransform,
}

impl QuadDrawQueue {
    pub fn new(dispatcher: Dispatcher) -> Self {
        QuadDrawQueue {
            dispatcher,
            queue: QueueBuilder::new(),
            textures: Vec::new(),
            current_texture: None,
            render_transform: RectTransform::identity(Rect::ZERO),
        }
    }
    pub fn start(&mut self, render_transform: RectTransform) {
        self.queue.reset();
        self.textures.clear();
        self.current_texture = None;
        self.render_transform = render_transform;
    }
    pub fn queue<T: ToQuad>(&mut self, item: &T) {
        let (quad, texture) = item.to_quad();
        let quad = if let Some(quad) = quad.transform_and_clip(&self.render_transform) {
            quad
        } else {
            return;
        };
        if self.current_texture.as_ref() != Some(&texture) {
            self.dispatch();
            self.current_texture = Some(texture);
        }
        self.queue.queue(quad);
    }
    pub fn dispatch(&mut self) {
        if let Some(texture) = self.current_texture.take() {
            self.queue.barrier();
            self.textures.push(texture);
            self.dispatcher.dispatch();
        }
    }
    pub fn draw_next(&mut self) -> (Option<Texture>, &[Quad]) {
        let (index, quads) = self.queue.draw_next();
        (self.textures[index].clone(), quads)
    }
}
