use std::any::Any;

use gristmill::color::Color;
use gristmill::geometry2d::Rect;
use super::{Widget, DrawContext, Drawable, GuiTexture};

pub struct Quad {
    pub color: Color,
    texture: Option<GuiTexture>,
    drawable: Option<Drawable>,
}

impl Quad {
    pub fn new_color(color: Color) -> Quad {
        Quad { color, texture: None, drawable: None }
    }
    pub fn new_texture(texture: GuiTexture) -> Quad {
        Quad { color: gristmill::color::white(), texture: Some(texture), drawable: None }
    }

    pub fn texture(&self) -> Option<&GuiTexture> { self.texture.as_ref() }
    pub fn set_texture(&mut self, texture: GuiTexture) {
        self.texture = Some(texture);
        self.drawable = None;
    }
    pub fn unset_texture(&mut self) {
        self.texture = None;
        self.drawable = None;
    }
}

impl Widget for Quad {
    fn as_any(&mut self) -> &mut dyn Any { self }
    fn draw(&mut self, context: &mut DrawContext, rect: Rect) {
        if self.drawable.is_none() {
            if let Some(texture) = self.texture.as_ref() {
                self.drawable = Some(context.new_texture_rect_drawable(texture.clone()));
            }
            else {
                self.drawable = Some(context.new_color_rect_drawable());
            }
        }
        context.draw(self.drawable.as_ref().unwrap(), rect, self.color);
    }
}
