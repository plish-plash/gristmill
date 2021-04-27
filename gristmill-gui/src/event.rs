use gristmill::event;
use gristmill::geometry2d::Point;
use super::GuiNode;

// -------------------------------------------------------------------------------------------------

pub enum GuiInputEvent {
    CursorMoved(Point),
    PrimaryButton(bool),
}

impl event::Event for GuiInputEvent {}

pub type GuiInputSystem = event::EventSystem<GuiInputEvent>;

pub enum GuiActionEvent {
    Action(String),
    Hover(GuiNode),
    Focus(GuiNode),
}

impl event::Event for GuiActionEvent {}

pub type GuiEventSystem = event::EventSystem<GuiActionEvent>;
