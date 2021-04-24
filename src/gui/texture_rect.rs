use std::any::Any;

use crate::color::Color;
use crate::geometry2d::Rect;
use super::{GuiNode, Widget, DrawContext, Drawable, Texture, GuiEventSystem, GuiInputEvent, GuiActionEvent};

pub struct TextureRect {
    pub color: Color,
    texture: Texture,
    drawable: Option<Drawable>,
}

impl TextureRect {
    pub fn new(texture: Texture) -> TextureRect {
        TextureRect { color: crate::color::white(), texture, drawable: None }
    }

    pub fn texture(&self) -> &Texture { &self.texture }
    pub fn set_texture(&mut self, texture: Texture) {
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
