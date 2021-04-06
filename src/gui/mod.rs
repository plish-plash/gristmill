pub mod font;

use crate::geometry2d::*;

pub use stretch::{node::Node, style::Style};
pub use super::renderer::subpass::gui::{DrawCommand, DrawContext, SizedDrawable, TextDrawable};

pub struct Gui;

impl Gui {
    pub fn new() -> Gui { Gui }

    pub fn root_node(&self) -> Node {
        unimplemented!();
    }
    pub fn children(&self, _node: Node) -> Option<Vec<Node>> {
        unimplemented!();
    }

    pub fn refresh_layout(&mut self, _screen_size: Size) {}
    pub fn refresh_drawables(&mut self, _context: &mut DrawContext) {}
    pub fn draw_widget(&self, _parent_position: Point, _node: Node) -> (Point, Option<DrawCommand>) {
        unimplemented!();
    }
}
