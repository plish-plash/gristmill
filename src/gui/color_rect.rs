use std::any::Any;

use crate::color::Color;
use crate::geometry2d::Rect;
use super::{GuiNode, Widget, DrawContext, Drawable, GuiEventSystem, GuiInputEvent, GuiActionEvent};

pub struct ColorRect {
    pub color: Color,
    drawable: Option<Drawable>,
}

impl ColorRect {
    pub fn new(color: Color) -> ColorRect {
        ColorRect { color, drawable: None }
    }
}

impl Widget for ColorRect {
    fn as_any(&mut self) -> &mut dyn Any { self }
    fn draw(&mut self, context: &mut DrawContext, rect: Rect) {
        if self.drawable.is_none() {
            self.drawable = Some(context.new_color_rect_drawable());
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
