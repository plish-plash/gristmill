use gristmill_core::geom2d::IRect;

use crate::NodeLayout;

pub trait GuiLayout {
    fn begin_layout(&mut self, rect: IRect, spacing: i32);
    fn layout_child(&mut self, layout: &NodeLayout) -> IRect;
}

#[derive(Default)]
pub struct Anchor(IRect);

impl GuiLayout for Anchor {
    fn begin_layout(&mut self, rect: IRect, _spacing: i32) {
        self.0 = rect;
    }
    fn layout_child(&mut self, layout: &NodeLayout) -> IRect {
        let (x, width) = layout.horizontal(self.0.x(), self.0.width());
        let (y, height) = layout.vertical(self.0.y(), self.0.height());
        IRect::new(x, y, width, height)
    }
}

#[derive(Default)]
pub struct HBox {
    rect: IRect,
    spacing: i32,
    x: i32,
}

impl GuiLayout for HBox {
    fn begin_layout(&mut self, rect: IRect, spacing: i32) {
        self.rect = rect;
        self.spacing = spacing;
        self.x = rect.position.x;
    }
    fn layout_child(&mut self, layout: &NodeLayout) -> IRect {
        let width = layout.width();
        let (y, height) = layout.vertical(self.rect.y(), self.rect.height());
        let child_rect = IRect::new(self.x, y, width, height);
        self.x += width + self.spacing;
        child_rect
    }
}

#[derive(Default)]
pub struct VBox {
    rect: IRect,
    spacing: i32,
    y: i32,
}

impl GuiLayout for VBox {
    fn begin_layout(&mut self, rect: IRect, spacing: i32) {
        self.rect = rect;
        self.spacing = spacing;
        self.y = rect.position.y;
    }
    fn layout_child(&mut self, layout: &NodeLayout) -> IRect {
        let height = layout.height();
        let (x, width) = layout.horizontal(self.rect.x(), self.rect.width());
        let child_rect = IRect::new(x, self.y, width, height);
        self.y += height + self.spacing;
        child_rect
    }
}
