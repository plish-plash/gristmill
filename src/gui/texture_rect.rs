use std::any::Any;

use crate::color::Color;
use crate::geometry2d::Rect;
use super::{GuiNode, Widget, DrawContext, Drawable, GuiTexture, GuiEventSystem, GuiInputEvent, GuiActionEvent};

pub struct TextureRect {
    pub color: Color,
    texture: GuiTexture,
    drawable: Option<Drawable>,
}

impl TextureRect {
    pub fn new(texture: GuiTexture) -> TextureRect {
        TextureRect { color: crate::color::white(), texture, drawable: None }
    }

    pub fn texture(&self) -> &GuiTexture { &self.texture }
    pub fn set_texture(&mut self, texture: GuiTexture) {
        self.texture = texture;
        self.drawable = None;
    }
}

impl Widget for TextureRect {
    fn as_any(&mut self) -> &mut dyn Any { self }
    fn draw(&mut self, context: &mut DrawContext, rect: Rect) {
        if self.drawable.is_none() {
            self.drawable = Some(context.new_texture_rect_drawable(self.texture.clone()));
        }
        context.draw(self.drawable.as_ref().unwrap(), rect, self.color);
    }
    fn handle_input(&mut self, node: GuiNode, event_system: &mut GuiEventSystem, input: GuiInputEvent) -> bool {
        match input {
            GuiInputEvent::CursorMoved(_) => event_system.fire_event(GuiActionEvent::Hover(node)),
            _ => (),
        }
        true
    }
}
