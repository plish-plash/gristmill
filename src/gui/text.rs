
use std::sync::Arc;
use super::*;

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
    font: Option<String>,
    size: f32,
    color: Color,
    align: (Align, Align),
    text: String,
    text_changed: bool,
    drawable: Option<Arc<TextDrawable>>,
}

impl Text {
    pub fn new() -> Text {
        Text {
            font: None,
            size: 14.,
            color: crate::color::black(),
            align: (Align::Start, Align::End),
            text: String::new(),
            text_changed: false,
            drawable: None,
        }
    }
    pub fn set_text_all(&mut self, font: Option<String>, size: f32, text: String) {
        self.font = font;
        self.size = size;
        self.text = text;
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
    fn refresh_drawables(&mut self, context: &mut DrawContext) {
        if self.text_changed {
            self.drawable = Some(context.text_drawable(self.font.as_deref(), self.size, &self.text));
            self.text_changed = false;
        }
    }
    fn minimum_size(&self) -> Option<Size> {
        self.drawable.as_ref().map(|drawable| {
            let height = if self.align.1 == Align::Start { 0 } else { drawable.ascent().ceil() as u32 };
            Size::new(drawable.width().ceil() as u32, height)
        })
    }
    fn draw(&self, rect: Rect) -> Option<DrawCommand> {
        self.drawable.as_ref().map(|drawable| {
            let x = self.align.0.position(rect.position.x as f32, rect.size.width as f32, drawable.width());
            let y = match self.align.1 {
                // Align baseline to container bottom.
                Align::End => self.align.1.position(rect.position.y as f32, rect.size.height as f32, 0.),
                // Align using the full height of the text.
                _ => self.align.1.position(rect.position.y as f32, rect.size.height as f32, drawable.height()) + drawable.ascent(),
            };
            drawable.clone().draw(Point::nearest(x, y), self.color)
        })
    }
}
