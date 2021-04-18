use std::sync::Arc;

use crate::color::Color;
use crate::geometry2d::Rect;
use super::{Widget, DrawContext, Drawable, SizedDrawable, GuiEventSystem, GuiInputEvent};

pub struct ColorRect {
    pub color: Color,
    drawable: Option<Arc<SizedDrawable>>,
}

impl ColorRect {
    pub fn new(color: Color) -> ColorRect {
        ColorRect { color, drawable: None }
    }
}

impl Widget for ColorRect {
    fn draw(&mut self, context: &mut DrawContext, rect: Rect) {
        if self.drawable.is_none() {
            self.drawable = Some(context.new_color_rect_drawable());
        }
        self.drawable.as_mut().unwrap().draw(context, rect, self.color);
    }
    fn handle_input(&mut self, _event_system: &mut GuiEventSystem, _input: &GuiInputEvent) -> bool { true }
}
