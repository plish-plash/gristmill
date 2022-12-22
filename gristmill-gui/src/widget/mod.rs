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

use crate::{Gui, GuiLayout, GuiNode, GuiNodeId, GuiNodeStorage};
use gristmill_core::input::ActionState;
use std::any::Any;

pub struct WidgetInput {
    pub state: ActionState,
    pub pointer_over: Option<GuiNodeId>,
}

pub trait Widget: Sized {
    fn type_name() -> &'static str;
    fn new(gui: &mut Gui, parent: GuiNodeId, style: StyleQuery) -> Self;
}

pub trait WidgetNode: 'static {
    fn as_any_box(self: Box<Self>) -> Box<dyn Any>;
    fn node(&self) -> GuiNodeId;
    fn node_data<'a>(&self, gui: &'a mut Gui) -> Option<&'a mut GuiNode> {
        gui.nodes.get_mut(self.node())
    }
    fn set_visible(&self, gui: &mut Gui, visible: bool) {
        if let Some(node) = self.node_data(gui) {
            node.flags.visible = visible;
        }
    }
    fn set_layout(&self, gui: &mut Gui, layout: GuiLayout) {
        if let Some(node) = self.node_data(gui) {
            node.layout = layout;
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
