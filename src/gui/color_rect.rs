
use std::sync::Arc;
use super::*;

pub struct ColorRect {
    color: Color,
    drawable: Option<Arc<SizedDrawable>>,
}

impl ColorRect {
    pub fn new(color: Color) -> ColorRect {
        ColorRect {
            color,
            drawable: None,
        }
    }
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

impl Widget for ColorRect {
    fn as_any(&mut self) -> &mut dyn Any { self }
    fn refresh_drawables(&mut self, context: &mut DrawContext) {
        if self.drawable.is_none() {
            self.drawable = Some(context.color_rect_drawable());
        }
    }
    fn draw(&self, rect: Rect) -> Option<DrawCommand> {
        self.drawable.as_ref().map(|drawable| {
            drawable.draw(rect, self.color)
        })
    }
}

pub struct ColorRectModifier {
    target: Node,
    color: Color,
}

impl ColorRectModifier {
    pub fn new(target: Node, color: Color) -> ColorRectModifier {
        ColorRectModifier { target, color }
    }
}

impl GuiModifier for ColorRectModifier {
    fn modify(&self, gui: &mut Gui) {
        let widget: &mut ColorRect = gui.widget_mut(self.target).expect("ColorRectModifier target is not a ColorRect");
        widget.set_color(self.color);
    }
}
