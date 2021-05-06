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
    padding: i32,
    pad_outside: bool,
}

impl BoxLayout {
    pub fn new(node: GuiNode, direction: BoxDirection, padding: i32) -> BoxLayout {
        BoxLayout {
            parent: node,
            direction_sides: direction.into_sides(),
            padding,
            pad_outside: false,
        }
    }
    pub fn set_pad_outside(&mut self, pad_outside: bool) {
        self.pad_outside = pad_outside;
    }

    fn layout(&self, first: bool, size: BoxSize) -> Layout {
        let (front_side, off_side) = self.direction_sides;
        let outside_padding = if self.pad_outside { self.padding } else { 0 };
        let size_dim = match size {
            BoxSize::Exact(v) => v,
            BoxSize::Remaining => 0,
        };
        let layout_size = match front_side {
            Side::Left => Size::new(size_dim, 0),
            _ => Size::new(0, size_dim),
        };
        let mut layout = Layout::with_base_size(layout_size);
        let outside_anchor = Anchor::parent(outside_padding);
        if first {
            layout.set_anchor(front_side, outside_anchor);
        }
        else {
            layout.set_anchor(front_side, Anchor::previous_sibling_opposite(self.padding));
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
    pub fn add_widget<W>(&self, gui: &mut Gui, widget: W, size: BoxSize) -> WidgetNode<W> where W: Widget + 'static {
        gui.add_widget(self.parent, widget, self.layout(!gui.has_children(self.parent), size))
    }
}

pub struct SplitLayout {
    parent: GuiNode,
    direction_sides: (Side, Side),
    padding: i32,
    pad_outside: bool,
}

impl SplitLayout {
    pub fn new(node: GuiNode, direction: BoxDirection, padding: i32) -> SplitLayout {
        SplitLayout {
            parent: node,
            direction_sides: direction.into_sides(),
            padding,
            pad_outside: false,
        }
    }
    pub fn set_pad_outside(&mut self, pad_outside: bool) {
        self.pad_outside = pad_outside;
    }

    fn layout(&self, first: bool, center_size: Option<u32>) -> Layout {
        let (front_side, off_side) = self.direction_sides;
        let outside_padding = if self.pad_outside { self.padding } else { 0 };
        let mut layout = Layout::with_base_size(center_size.map(|dim| {
            match front_side {
                Side::Left => Size::new(dim, 0),
                _ => Size::new(0, dim),
            }
        }).unwrap_or_default());
        let outside_anchor = Anchor::parent(outside_padding);
        if let Some(size) = center_size {
            layout.set_anchor(front_side, Anchor::parent_center(-(size as i32) / 2));
        }
        else {
            if first {
                layout.set_anchor(front_side, outside_anchor);
                layout.set_anchor(front_side.opposite(), Anchor::parent_center(-self.padding / 2));
            }
            else {
                layout.set_anchor(front_side, Anchor::parent_center(self.padding / 2));
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
        gui.add_widget(self.parent, widget, self.layout(!gui.has_children(self.parent), None))
    }
    pub fn add_center(&self, gui: &mut Gui, size: u32) -> GuiNode {
        gui.add(self.parent, self.layout(false, Some(size)))
    }
    pub fn add_center_widget<W>(&self, gui: &mut Gui, widget: W, size: u32) -> WidgetNode<W> where W: Widget + 'static {
        gui.add_widget(self.parent, widget, self.layout(false, Some(size)))
    }
}
