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

use crate::{Gui, GuiLayout, GuiNodeKey, GuiNodeObj};
use gristmill::input::ActionState;

#[derive(Copy, Clone)]
pub struct WidgetInput<'a> {
    pub state: &'a ActionState,
    pub pointer_over: Option<GuiNodeKey>,
}

pub trait WidgetState: 'static {
    fn update(&mut self, input: WidgetInput);
}

pub trait Widget: Sized {
    fn class_name() -> &'static str;
    fn new(gui: &mut Gui, parent: GuiNodeObj) -> Self;
    fn apply_style(&mut self, _style: StyleQuery) {}

    fn node(&self) -> &GuiNodeObj;
    fn set_visible(&self, visible: bool) {
        self.node().write().flags.visible = visible;
    }
    fn set_layout(&self, layout: GuiLayout) {
        self.node().write().layout = layout;
    }
}
