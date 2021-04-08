pub mod font;

use slotmap::new_key_type;

use crate::geometry2d::*;
use crate::forest::Forest;

pub use super::renderer::subpass::gui::{DrawCommand, DrawContext, SizedDrawable, TextDrawable};

new_key_type! {
    pub struct GuiNode;
}

struct GuiItem {

}

pub struct Gui {
    forest: Forest<GuiNode, GuiItem>,
    render_root: GuiNode,
}

impl Gui {
    pub fn new() -> Gui {
        let mut forest = Forest::new();
        let render_root = forest.add(GuiItem {});
        Gui { forest, render_root }
    }

    pub fn root(&self) -> GuiNode {
        self.render_root
    }
    pub fn get_children(&self, node: GuiNode) -> Vec<GuiNode> {
        self.forest.get_children(node)
    }
    pub fn iter_children(&self, node: GuiNode) -> std::slice::Iter<'_, GuiNode> {
        self.forest.iter_children(node)
    }

    pub fn refresh_layout(&mut self, _screen_size: Size) {}
    pub fn refresh_drawables(&mut self, _context: &mut DrawContext) {}
    pub fn draw_widget(&self, _parent_position: Point, _node: GuiNode) -> (Point, Option<DrawCommand>) {
        unimplemented!();
    }
}
