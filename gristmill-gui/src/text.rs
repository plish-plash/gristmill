use std::any::Any;

use gristmill::color::Color;
use gristmill::geometry2d::*;
use super::{Widget, DrawContext, Drawable, TextMetrics, font::Font};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Align {
    Start,
    Middle,
    End,
}

impl Align {
    fn position(self, outer_pos: f32, outer_size: f32, inner_size: f32) -> f32 {
        match self {
            Align::Start => outer_pos,
            Align::Middle => outer_pos + (outer_size / 2.) - (inner_size / 2.),
            Align::End => outer_pos + outer_size - inner_size,
        }
    }
}

pub struct Text {
    font: Font,
    size: f32,
    color: Color,
    align: (Align, Align),
    text: String,
    text_changed: bool,
    drawable: Option<(Drawable, TextMetrics)>,
}

impl Text {
    pub fn new(text: String) -> Text {
        let text_changed = !text.is_empty();
        Text {
            font: Font::default(),
            size: 14.,
            color: gristmill::color::black(),
            align: (Align::Start, Align::Start),
            text,
            text_changed,
            drawable: None,
        }
    }
    pub fn new_empty() -> Text {
        Text::new(String::new())
    }
    pub fn set_font(&mut self, font: Font, size: f32) {
        self.font = font;
        self.size = size;
        self.text_changed = true;
    }
    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.text_changed = true;
    }
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    pub fn set_alignment(&mut self, h_align: Align, v_align: Align) {
        self.align = (h_align, v_align)
    }
}

impl Widget for Text {
    fn as_any(&mut self) -> &mut dyn Any { self }
    fn draw(&mut self, context: &mut DrawContext, rect: Rect) {
        if self.text_changed {
            self.drawable = Some(context.new_text_drawable(self.font, self.size, &self.text));
            self.text_changed = false;
        }
        if let Some((drawable, metrics)) = self.drawable.as_ref() {
            let x = self.align.0.position(rect.position.x as f32, rect.size.width as f32, metrics.width());
            let y = match self.align.1 {
                // Align baseline to container bottom.
                Align::End => self.align.1.position(rect.position.y as f32, rect.size.height as f32, 0.),
                // Align using the full height of the text.
                _ => self.align.1.position(rect.position.y as f32, rect.size.height as f32, metrics.height()) + metrics.ascent(),
            };
            context.draw(drawable, Rect { position: Point::nearest(x, y), size: Size::zero() }, self.color);
        }
    }
}