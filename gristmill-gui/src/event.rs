use gristmill::event;
use gristmill::geometry2d::Point;
use super::GuiNode;

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub enum GuiInputEvent {
    CursorMoved(Point),
    PrimaryButton(bool),
}

impl event::Event for GuiInputEvent {}

pub type GuiInputSystem = event::EventSystem<GuiInputEvent>;

#[derive(Clone)]
pub enum GuiActionEvent {
    Generic,
    Named(String),
    Index(usize),
    NamedIndex(String, usize),
}

impl event::Event for GuiActionEvent {}

// This type is a convenience for pattern-matching
#[derive(Copy, Clone)]
pub enum GuiActionEventRef<'a> {
    Generic,
    Named(&'a str),
    Index(usize),
    NamedIndex(&'a str, usize),
}

impl GuiActionEvent {
    pub fn as_ref(&self) -> GuiActionEventRef {
        match self {
            GuiActionEvent::Generic => GuiActionEventRef::Generic,
            GuiActionEvent::Named(s) => GuiActionEventRef::Named(s),
            GuiActionEvent::Index(i) => GuiActionEventRef::Index(*i),
            GuiActionEvent::NamedIndex(s, i) => GuiActionEventRef::NamedIndex(s, *i),
        }
    }
}

pub enum GuiNavigationEvent {
    Hover(GuiNode),
    Focus(GuiNode),
}

impl event::Event for GuiNavigationEvent {}

pub type GuiActionEventSystem = event::EventSystem<GuiActionEvent>;
pub type GuiNavigationEventSystem = event::EventSystem<GuiNavigationEvent>;

pub struct GuiEventSystem<'a> {
    action_system: Option<&'a mut GuiActionEventSystem>,
    navigation_system: &'a mut GuiNavigationEventSystem,
}

impl<'a> GuiEventSystem<'a> {
    pub(crate) fn new(action_system: Option<&'a mut GuiActionEventSystem>, navigation_system: &'a mut GuiNavigationEventSystem) -> GuiEventSystem<'a> {
        GuiEventSystem { action_system, navigation_system }
    }
    pub fn fire_action(&mut self, event: GuiActionEvent) {
        if let Some(action_system) = &mut self.action_system {
            action_system.fire_event(event);
        }
    }
    pub fn fire_navigation(&mut self, event: GuiNavigationEvent) {
        self.navigation_system.fire_event(event);
    }
}
