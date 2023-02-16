mod button;
mod image;
mod panel;
mod style;
mod text;

pub use button::*;
pub use image::*;
pub use panel::*;
pub use style::*;
pub use text::*;

use crate::{Gui, GuiNode, GuiNodeId, GuiNodeStorage};
use gristmill_core::{geom2d::EdgeRect, input::ActionState, math::IVec2};
use std::any::Any;

pub struct WidgetInput {
    pub state: ActionState,
    pub pointer_over: Option<GuiNodeId>,
}

pub trait Widget: Sized {
    fn class_name() -> &'static str;
    fn new(gui: &mut Gui, parent: GuiNodeId, style: StyleValues) -> Self;
}

pub trait WidgetNode: 'static {
    fn as_any_box(self: Box<Self>) -> Box<dyn Any>;
    fn node(&self) -> GuiNodeId;
}

pub trait WidgetNodeExt {
    fn node_data<'a>(&self, gui: &'a mut Gui) -> Option<&'a mut GuiNode>;
    fn set_visible(&self, gui: &mut Gui, visible: bool);
    fn set_child_layout<S: Into<String>>(&self, gui: &mut Gui, layout: S);
    fn set_child_spacing(&self, gui: &mut Gui, spacing: i32);
    fn set_layout_size(&self, gui: &mut Gui, size: IVec2);
    fn set_layout_margin(&self, gui: &mut Gui, margin: EdgeRect);
    fn set_layout_width(&self, gui: &mut Gui, width: i32);
    fn set_layout_height(&self, gui: &mut Gui, height: i32);
}
impl<T: WidgetNode> WidgetNodeExt for T {
    fn node_data<'a>(&self, gui: &'a mut Gui) -> Option<&'a mut GuiNode> {
        gui.nodes.get_mut(self.node())
    }
    fn set_visible(&self, gui: &mut Gui, visible: bool) {
        if let Some(node) = self.node_data(gui) {
            node.flags.visible = visible;
        }
    }
    fn set_child_layout<S: Into<String>>(&self, gui: &mut Gui, layout: S) {
        if let Some(node) = self.node_data(gui) {
            node.layout.child_layout = layout.into();
        }
    }
    fn set_child_spacing(&self, gui: &mut Gui, spacing: i32) {
        if let Some(node) = self.node_data(gui) {
            node.layout.child_spacing = spacing;
        }
    }
    fn set_layout_size(&self, gui: &mut Gui, size: IVec2) {
        if let Some(node) = self.node_data(gui) {
            node.layout.size = size;
        }
    }
    fn set_layout_margin(&self, gui: &mut Gui, margin: EdgeRect) {
        if let Some(node) = self.node_data(gui) {
            node.layout.margin = margin;
        }
    }
    fn set_layout_width(&self, gui: &mut Gui, width: i32) {
        if let Some(node) = self.node_data(gui) {
            node.layout.size.x = width;
        }
    }
    fn set_layout_height(&self, gui: &mut Gui, height: i32) {
        if let Some(node) = self.node_data(gui) {
            node.layout.size.y = height;
        }
    }
}

impl WidgetNode for GuiNodeId {
    fn as_any_box(self: Box<Self>) -> Box<dyn Any> {
        self
    }
    fn node(&self) -> GuiNodeId {
        *self
    }
}

pub trait WidgetBehavior: 'static {
    fn update(&self, nodes: &mut GuiNodeStorage, input: &WidgetInput);
}
