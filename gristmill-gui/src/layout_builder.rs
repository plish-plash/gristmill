use gristmill::geometry2d::*;
use super::{Gui, GuiNode, WidgetNode, Widget, layout::*};

pub enum BoxSize {
    Exact(u32),
    Remaining,
}

pub enum BoxDirection {
    Horizontal,
    Vertical,
}

impl BoxDirection {
    fn into_sides(self) -> (Side, Side) {
        match self {
            BoxDirection::Horizontal => (Side::Left, Side::Top),
            BoxDirection::Vertical => (Side::Top, Side::Left),
        }
    }
}

pub struct BoxLayout {
    parent: GuiNode,
    direction_sides: (Side, Side),
    padding: Padding,
}

impl BoxLayout {
    pub fn new(node: GuiNode, direction: BoxDirection, padding: Padding) -> BoxLayout {
        BoxLayout {
            parent: node,
            direction_sides: direction.into_sides(),
            padding,
        }
    }

    fn layout(&self, first: bool, size: BoxSize) -> Layout {
        let (front_side, off_side) = self.direction_sides;
        let size_dim = match size {
            BoxSize::Exact(v) => v,
            BoxSize::Remaining => 0,
        };
        let layout_size = match front_side {
            Side::Left => Size::new(size_dim, 0),
            _ => Size::new(0, size_dim),
        };
        let mut layout = Layout::with_base_size(layout_size);
        let outside_anchor = Anchor::parent(self.padding.outside());
        if first {
            layout.set_anchor(front_side, outside_anchor);
        }
        else {
            layout.set_anchor(front_side, Anchor::previous_sibling_opposite(self.padding.inside()));
        }
        layout.set_anchor(off_side, outside_anchor);
        layout.set_anchor(off_side.opposite(), outside_anchor);
        if let BoxSize::Remaining = size {
            layout.set_anchor(front_side.opposite(), outside_anchor);
        }
        layout
    }
    pub fn add(&self, gui: &mut Gui, size: BoxSize) -> GuiNode {
        gui.add(self.parent, self.layout(!gui.has_children(self.parent), size))
    }
    pub fn add_widget<W>(&self, gui: &mut Gui, size: BoxSize, widget: W) -> WidgetNode<W> where W: Widget + 'static {
        gui.add_widget(self.parent, self.layout(!gui.has_children(self.parent), size), widget)
    }
}

pub struct SplitLayout {
    parent: GuiNode,
    direction_sides: (Side, Side),
    padding: Padding,
}

impl SplitLayout {
    pub fn new(node: GuiNode, direction: BoxDirection, padding: Padding) -> SplitLayout {
        SplitLayout {
            parent: node,
            direction_sides: direction.into_sides(),
            padding,
        }
    }

    fn layout(&self, first: bool, center_size: Option<u32>) -> Layout {
        let (front_side, off_side) = self.direction_sides;
        let mut layout = Layout::with_base_size(center_size.map(|dim| {
            match front_side {
                Side::Left => Size::new(dim, 0),
                _ => Size::new(0, dim),
            }
        }).unwrap_or_default());
        let outside_anchor = Anchor::parent(self.padding.outside());
        if let Some(size) = center_size {
            layout.set_anchor(front_side, Anchor::parent_center(-(size as i32) / 2));
        }
        else {
            let inside_anchor = Anchor::parent_center(self.padding.inside() / 2);
            if first {
                layout.set_anchor(front_side, outside_anchor);
                layout.set_anchor(front_side.opposite(), inside_anchor);
            }
            else {
                layout.set_anchor(front_side, inside_anchor);
                layout.set_anchor(front_side.opposite(), outside_anchor);
            }
        }
        layout.set_anchor(off_side, outside_anchor);
        layout.set_anchor(off_side.opposite(), outside_anchor);
        layout
    }
    pub fn add(&self, gui: &mut Gui) -> GuiNode {
        gui.add(self.parent, self.layout(!gui.has_children(self.parent), None))
    }
    pub fn add_widget<W>(&self, gui: &mut Gui, widget: W) -> WidgetNode<W> where W: Widget + 'static {
        gui.add_widget(self.parent, self.layout(!gui.has_children(self.parent), None), widget)
    }
    pub fn add_center(&self, gui: &mut Gui, size: u32) -> GuiNode {
        gui.add(self.parent, self.layout(false, Some(size)))
    }
    pub fn add_center_widget<W>(&self, gui: &mut Gui, size: u32, widget: W) -> WidgetNode<W> where W: Widget + 'static {
        gui.add_widget(self.parent, self.layout(false, Some(size)), widget)
    }
}
