pub mod color_rect;
pub mod font;
pub mod layout;
pub mod text;

use std::cell::Cell;

use slotmap::{new_key_type, SecondaryMap};

use crate::geometry2d::*;
use crate::forest::Forest;

use layout::Layout;

pub use super::renderer::subpass::gui::{DrawContext, Drawable, SizedDrawable, TextDrawable};

new_key_type! {
    pub struct GuiNode;
}

pub trait Widget {
    fn draw(&mut self, context: &mut DrawContext, rect: Rect);
}

struct GuiItem {
    rect: Cell<Rect>,
    layout: Layout,
}

impl GuiItem {
    fn new() -> GuiItem {
        GuiItem { rect: Cell::default(), layout: Layout::default() }
    }
    fn with_layout(layout: Layout) -> GuiItem {
        GuiItem { rect: Cell::default(), layout }
    }
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
            let child_rect = forest.get(*child).rect.get();
            self.draw_node(forest, context, *child, child_rect);
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
        let render_root = forest.add(GuiItem::new());
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

    pub fn add<W>(&mut self, parent: GuiNode, layout: Layout, widget: W) -> GuiNode where W: Widget + 'static {
        let node = self.forest.add_child(parent, GuiItem::with_layout(layout));
        self.widgets.insert(node, widget);
        node
    }
    pub fn set_node_layout(&mut self, node: GuiNode, layout: Layout) {
        let item = self.forest.get_mut(node);
        item.layout = layout;
    }

    pub fn draw(&mut self, context: &mut DrawContext) {
        let root_rect = self.forest.get(self.render_root).rect.get();
        self.widgets.draw_node(&self.forest, context, self.render_root, root_rect);
    }

    pub fn layout_if_needed(&self, parent_size: Size) {
        let root_rect = self.forest.get(self.render_root).rect.get();
        if root_rect.size != parent_size {
            self.layout(self.render_root, Rect { position: Point::origin(), size: parent_size }, None);
        }
    }
    fn layout(&self, node: GuiNode, parent_rect: Rect, previous_sibling_rect: Option<Rect>) -> Rect {
        let item = self.forest.get(node);
        let rect = if node == self.render_root {
            parent_rect
        } else {
            let context = LayoutContext { parent_rect, previous_sibling_rect };
            item.layout.layout_before_children(&context)
        };
        item.rect.set(rect);
        let mut previous_child_rect = None;
        for child in self.iter_children(node) {
            let child_rect = self.layout(*child, rect, previous_child_rect);
            previous_child_rect = Some(child_rect);
        }
        rect
    }
}

pub struct LayoutContext {
    parent_rect: Rect,
    previous_sibling_rect: Option<Rect>,
}

impl LayoutContext {
    pub fn parent_rect(&self) -> Rect {
        self.parent_rect
    }
    pub fn previous_sibling_rect(&self) -> Rect {
        self.previous_sibling_rect.expect("first child can't have PreviousSibling anchor")
    }
}
