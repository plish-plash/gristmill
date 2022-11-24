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

use crate::{Gui, GuiLayout, GuiNode};
use downcast_rs::{impl_downcast, Downcast};
use gristmill::input::ActionState;
use gristmill::{CastObj, Obj};

#[derive(Copy, Clone)]
pub struct WidgetInput<'a> {
    pub state: &'a ActionState,
    pub pointer_over: &'a Option<Obj<GuiNode>>,
}

pub trait WidgetBehavior: Downcast {
    fn update(&mut self, input: WidgetInput);
}
impl_downcast!(WidgetBehavior);

pub trait Widget: Sized {
    fn class_name() -> &'static str;
    fn new(gui: &mut Gui, parent: Obj<GuiNode>) -> Self;
    fn apply_style(&mut self, _style: StyleQuery) {}

    fn node(&self) -> Obj<GuiNode>;
    fn set_visible(&self, visible: bool) {
        self.node().write().flags.visible = visible;
    }
    fn set_layout(&self, layout: GuiLayout) {
        self.node().write().layout = layout;
    }
}

pub type WidgetObj<T> = CastObj<dyn WidgetBehavior, T>;
