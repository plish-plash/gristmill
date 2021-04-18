use crate::event;

use super::GuiNode;

// -------------------------------------------------------------------------------------------------

pub enum GuiInputEvent {
    CursorMoved(f32, f32),
    PrimaryButton,
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
