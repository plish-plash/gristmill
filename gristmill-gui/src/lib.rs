pub mod button;
pub mod container;
pub mod event;
pub mod font;
pub mod layout;
pub mod layout_builder;
pub mod listener;
pub mod text;
pub mod quad;
pub mod renderer;

use std::any::Any;
use std::cell::Cell;

use slotmap::{new_key_type, Key, SecondaryMap};

use gristmill::geometry2d::*;
use gristmill::util::forest::Forest;
use gristmill::input::CursorAction;

use layout::Layout;
use event::{GuiActionEvent, GuiActionEventSystem, GuiNavigationEvent, GuiNavigationEventSystem};

pub use container::Container;
pub use listener::GuiValue;
pub use event::{GuiEventSystem, GuiInputEvent};
pub use renderer::{DrawContext, Drawable, GuiTexture, TextMetrics};

#[macro_export]
macro_rules! impl_class_field_fn {
    ($field:ident -> $field_type:ty) => {
        fn $field(&self) -> $field_type {
            if self.$field.is_some() {
                self.$field.as_ref()
            }
            else if let Some(parent) = self.parent.as_ref() {
                parent.$field()
            }
            else { None }
        }
    };
}

new_key_type! {
    pub struct GuiNode;
}

pub trait Widget {
    fn as_any(&mut self) -> &mut dyn Any;
    fn draw(&mut self, context: &mut DrawContext, rect: Rect);
    fn handle_input(&mut self, _node: GuiNode, _event_system: GuiEventSystem, _input: GuiInputEvent) -> bool { false }
    fn set_hovered(&mut self, _hovered: bool) {}
    fn set_focused(&mut self, _focused: bool) {}
}

// Type-safe GuiNode
#[derive(Eq, PartialEq, Debug, Hash)]
pub struct WidgetNode<W: Widget> {
    node: GuiNode,
    marker: std::marker::PhantomData<W>,
}
impl<W: Widget> WidgetNode<W> {
    fn new(node: GuiNode) -> WidgetNode<W> {
        WidgetNode { node, marker: std::marker::PhantomData }
    }
}
// Can't derive because of the type parameter
impl<W: Widget> Copy for WidgetNode<W> { }
impl<W: Widget> Clone for WidgetNode<W> {
    fn clone(&self) -> WidgetNode<W> { WidgetNode::new(self.node) }
}
impl<W: Widget> From<WidgetNode<W>> for GuiNode {
    fn from(node: WidgetNode<W>) -> GuiNode { node.node }
}

struct GuiItem {
    rect: Cell<Rect>,
    layout: Layout,
    event_handler: Option<GuiNode>,
}

impl GuiItem {
    fn new() -> GuiItem {
        GuiItem { rect: Cell::default(), layout: Layout::default(), event_handler: None }
    }
    fn with_layout(layout: Layout) -> GuiItem {
        GuiItem { rect: Cell::default(), layout, event_handler: None }
    }
}

struct GuiWidgets {
    widgets: SecondaryMap<GuiNode, Box<dyn Widget>>,
    event_handlers: SecondaryMap<GuiNode, GuiActionEventSystem>,
}

impl GuiWidgets {
    fn new() -> GuiWidgets {
        GuiWidgets { widgets: SecondaryMap::new(), event_handlers: SecondaryMap::new() }
    }
    fn insert<W>(&mut self, node: GuiNode, widget: W) where W: Widget + 'static {
        self.widgets.insert(node, Box::new(widget));
    }
    fn draw_node(&mut self, forest: &Forest<GuiNode, GuiItem>, node: GuiNode, context: &mut DrawContext, rect: Rect) {
        if let Some(widget) = self.widgets.get_mut(node) {
            widget.draw(context, rect);
        }
        for child in forest.iter_children(node) {
            let child_rect = forest.get(*child).rect.get();
            self.draw_node(forest, *child, context, child_rect);
        }
    }
    fn handle_input(&mut self, forest: &Forest<GuiNode, GuiItem>, node: GuiNode, event_system: &mut GuiNavigationEventSystem, input: GuiInputEvent) -> bool {
        if let Some(widget) = self.widgets.get_mut(node) {
            let event_handler = if let Some(handler) = forest.get(node).event_handler {
                self.event_handlers.get_mut(handler)
            } else { None };
            widget.handle_input(node, GuiEventSystem::new(event_handler, event_system), input)
        }
        else { false }
    }
    fn handle_cursor_moved(&mut self, forest: &Forest<GuiNode, GuiItem>, node: GuiNode, event_system: &mut GuiNavigationEventSystem, cursor_pos: Point) -> bool {
        for child in forest.iter_children(node) {
            if self.handle_cursor_moved(forest, *child, event_system, cursor_pos) {
                return true;
            }
        }
        let rect = forest.get(node).rect.get();
        if rect.contains(cursor_pos) {
            let relative_pos = cursor_pos.relative_to(rect.position);
            self.handle_input(forest, node, event_system, GuiInputEvent::CursorMoved(relative_pos))
        }
        else { false }
    }
}

struct GuiContainers {
    containers: SecondaryMap<GuiNode, Box<dyn Container>>,
}

impl GuiContainers {
    fn new() -> GuiContainers {
        GuiContainers { containers: SecondaryMap::new() }
    }
    fn insert<C>(&mut self, node: GuiNode, container: C) where C: Container + 'static {
        self.containers.insert(node, Box::new(container));
    }
    fn layout(&mut self, forest: &Forest<GuiNode, GuiItem>, node: GuiNode, item: &GuiItem, rect: Rect) {
        item.rect.set(rect);
        let mut child_index = 0;
        let mut previous_child_rect = None;
        for child in forest.iter_children(node) {
            let child_item = forest.get(*child);
            let child_layout = if let Some(container) = self.containers.get_mut(node) {
                container.layout_child(rect, child_index, child_item.layout.base_size)
            } else {
                child_item.layout.clone()
            };
            let context = LayoutContext { parent_rect: rect, previous_sibling_rect: previous_child_rect };
            let child_rect = child_layout.layout_self(&context);
            self.layout(forest, *child, child_item, child_rect);
            child_index += 1;
            previous_child_rect = Some(child_rect);
        }
    }
}

pub trait GuiInputActions {
    fn primary(&self) -> &CursorAction;
}

struct GuiInputState {
    last_cursor_pos: Point,
    hovered: GuiNode,
    focused: GuiNode,
}

impl GuiInputState {
    fn new() -> GuiInputState {
        let last_cursor_pos = Point::new(-1, -1);
        GuiInputState { last_cursor_pos, hovered: GuiNode::null(), focused: GuiNode::null() }
    }
    fn handle_event(&mut self, widgets: &mut GuiWidgets, event: &GuiNavigationEvent) {
        match event {
            GuiNavigationEvent::Hover(node) => self.set_hovered(widgets, *node),
            GuiNavigationEvent::Focus(_node) => unimplemented!(),
        }
    }
    fn set_hovered(&mut self, widgets: &mut GuiWidgets, node: GuiNode) {
        if self.hovered != node {
            if let Some(widget) = widgets.widgets.get_mut(self.hovered) {
                widget.set_hovered(false);
            }
            self.hovered = node;
            if let Some(widget) = widgets.widgets.get_mut(self.hovered) {
                widget.set_hovered(true);
            }
        }
    }
}

pub struct Gui {
    forest: Forest<GuiNode, GuiItem>,
    widgets: GuiWidgets,
    containers: GuiContainers,
    input_state: GuiInputState,
    render_root: GuiNode,
    navigation_events: GuiNavigationEventSystem,
}

impl Gui {
    pub fn new() -> Gui {
        let mut forest = Forest::new();
        let render_root = forest.add(GuiItem::new());
        Gui {
            forest,
            input_state: GuiInputState::new(),
            widgets: GuiWidgets::new(),
            containers: GuiContainers::new(),
            render_root,
            navigation_events: GuiNavigationEventSystem::new(),
        }
    }

    pub fn root(&self) -> GuiNode {
        self.render_root
    }
    pub fn has_children(&self, node: GuiNode) -> bool {
        self.forest.get_child_count(node) > 0
    }
    pub fn get_children(&self, node: GuiNode) -> Vec<GuiNode> {
        self.forest.get_children(node)
    }
    pub fn iter_children(&self, node: GuiNode) -> std::slice::Iter<'_, GuiNode> {
        self.forest.iter_children(node)
    }

    // pub fn get<W>(&self, node: WidgetNode<W>) -> Option<&W> where W: Widget + 'static {
    //     self.widgets.widgets.get(node.into()).map(|w| {
    //         w.as_any().downcast_ref::<W>().unwrap()
    //     })
    // }
    pub fn get_mut<W>(&mut self, node: WidgetNode<W>) -> Option<&mut W> where W: Widget + 'static {
        self.widgets.widgets.get_mut(node.into()).map(|w| {
            w.as_any().downcast_mut::<W>().unwrap()
        })
    }
    pub fn get_events(&mut self, node: GuiNode) -> Option<&mut GuiActionEventSystem> {
        self.widgets.event_handlers.get_mut(node)
    }

    pub fn add(&mut self, parent: GuiNode, layout: Layout) -> GuiNode {
        let mut item = GuiItem::with_layout(layout);
        if let Some(event_handler) = self.forest.get(parent).event_handler {
            item.event_handler = Some(event_handler);
        }
        self.forest.add_child(parent, item)
    }
    pub fn add_widget<W>(&mut self, parent: GuiNode, layout: Layout, widget: W) -> WidgetNode<W> where W: Widget + 'static {
        let node = self.add(parent, layout);
        self.widgets.insert(node, widget);
        WidgetNode::new(node)
    }
    pub fn set_size(&mut self, node: GuiNode, size: Size) {
        let item = self.forest.get_mut(node);
        item.layout.base_size = size;
    }
    pub fn set_layout(&mut self, node: GuiNode, layout: Layout) {
        let item = self.forest.get_mut(node);
        item.layout = layout;
    }
    pub fn set_container<C>(&mut self, node: GuiNode, container: C) where C: Container + 'static {
        self.containers.insert(node, container);
    }
    pub fn set_event_handler(&mut self, node: GuiNode) {
        self.forest.get_mut(node).event_handler = Some(node);
        self.widgets.event_handlers.insert(node, GuiActionEventSystem::new());
    }

    fn draw(&mut self, context: &mut DrawContext) {
        let root_rect = self.forest.get(self.render_root).rect.get();
        self.widgets.draw_node(&self.forest, self.render_root, context, root_rect);
    }

    fn layout_if_needed(&mut self, parent_size: Size) {
        let root_rect = self.forest.get(self.render_root).rect.get();
        if root_rect.size != parent_size {
            self.containers.layout(
                &self.forest,
                self.render_root,
                self.forest.get(self.render_root),
                Rect::from_size(parent_size)
            );
        }
    }

    fn fire_input(&mut self, input: GuiInputEvent) {
        match input {
            GuiInputEvent::CursorMoved(cursor_pos) => {
                if !self.widgets.handle_cursor_moved(&self.forest, self.render_root, &mut self.navigation_events, cursor_pos) {
                    // If the cursor isn't over anything, set the hovered widget to null.
                    self.input_state.set_hovered(&mut self.widgets, GuiNode::null());
                }
            }
            GuiInputEvent::PrimaryButton(_) => {
                self.widgets.handle_input(&self.forest, self.input_state.hovered, &mut self.navigation_events, input);
            }
        }
    }
    pub fn process_input<A>(&mut self, actions: &A) where A: GuiInputActions {
        // Convert input actions to GuiInputEvent and send them to relevant widgets.
        // Widgets respond to GuiInputEvents by sending GuiActionEvents and GuiNavigationEvents to the event system.
        let primary_action = actions.primary();
        let cursor_pos = primary_action.position();
        if cursor_pos != self.input_state.last_cursor_pos {
            self.input_state.last_cursor_pos = cursor_pos;
            self.fire_input(GuiInputEvent::CursorMoved(cursor_pos));
        }
        if primary_action.pressed() {
            self.fire_input(GuiInputEvent::PrimaryButton(true));
        }
        else if primary_action.released() {
            self.fire_input(GuiInputEvent::PrimaryButton(false));
        }

        // Process the resulting GuiNavigationEvents.
        let widgets = &mut self.widgets;
        let input_state = &mut self.input_state;
        self.navigation_events.dispatch_queue(move |event| {
            input_state.handle_event(widgets, &event);
        });
    }
}

pub struct LayoutContext {
    parent_rect: Rect,
    previous_sibling_rect: Option<Rect>,
}

impl LayoutContext {
    pub fn parent_rect(&self) -> Rect {
        self.parent_rect
    }
    pub fn previous_sibling_rect(&self) -> Rect {
        self.previous_sibling_rect.expect("first child can't have PreviousSibling anchor")
    }
}
