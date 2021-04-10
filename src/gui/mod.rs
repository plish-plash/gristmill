pub mod color_rect;
pub mod font;

use slotmap::{new_key_type, SecondaryMap};

use crate::geometry2d::*;
use crate::forest::Forest;

pub use super::renderer::subpass::gui::{DrawContext, Drawable, SizedDrawable, TextDrawable};

new_key_type! {
    pub struct GuiNode;
}

pub trait Widget {
    fn draw(&mut self, context: &mut DrawContext, rect: Rect);
}

struct GuiItem {
    rect: Rect, // TODO
}

struct GuiWidgets {
    widgets: SecondaryMap<GuiNode, Box<dyn Widget>>,
}

impl GuiWidgets {
    fn new() -> GuiWidgets {
        GuiWidgets { widgets: SecondaryMap::new() }
    }
    fn insert<W>(&mut self, node: GuiNode, widget: W) where W: Widget + 'static {
        self.widgets.insert(node, Box::new(widget));
    }
    fn draw_node(&mut self, forest: &Forest<GuiNode, GuiItem>, context: &mut DrawContext, node: GuiNode, rect: Rect) {
        if let Some(widget) = self.widgets.get_mut(node) {
            widget.draw(context, rect);
        }
        for child in forest.iter_children(node) {
            let rect = forest.get(*child).rect;
            self.draw_node(forest, context, *child, rect);
        }
    }
}

pub struct Gui {
    forest: Forest<GuiNode, GuiItem>,
    widgets: GuiWidgets,
    render_root: GuiNode,
}

impl Gui {
    pub fn new() -> Gui {
        let mut forest = Forest::new();
        let render_root = forest.add(GuiItem { rect: Rect::zero() });
        Gui { forest, widgets: GuiWidgets::new(), render_root }
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

    pub fn add<W>(&mut self, parent: GuiNode, widget: W) -> GuiNode where W: Widget + 'static {
        let node = self.forest.add_child(parent, GuiItem { rect: Rect::zero() });
        self.widgets.insert(node, widget);
        node
    }
    pub fn set_node_rect(&mut self, node: GuiNode, rect: Rect) {
        let item = self.forest.get_mut(node);
        item.rect = rect;
    }

    pub fn draw(&mut self, context: &mut DrawContext) {
        let root_rect = self.forest.get(self.render_root).rect;
        self.widgets.draw_node(&self.forest, context, self.render_root, root_rect);
    }

    pub fn layout_if_needed(&mut self, parent_size: Size) {
        // TODO
        let root_item = self.forest.get_mut(self.render_root);
        root_item.rect = Rect { position: Point::origin(), size: parent_size };
    }
    // fn layout(&mut self, layout_root: GuiNode, parent_size: Size) {
    // }
}
