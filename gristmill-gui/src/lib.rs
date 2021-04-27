pub mod button;
pub mod color_rect;
pub mod event;
pub mod font;
pub mod layout;
pub mod text;
pub mod texture_rect;
pub mod renderer;

use std::any::Any;
use std::cell::Cell;

use slotmap::{new_key_type, Key, SecondaryMap};

use gristmill::geometry2d::*;
use gristmill::util::forest::Forest;
use gristmill::input::CursorAction;

use layout::Layout;

pub use event::*;
pub use renderer::{DrawContext, Drawable, GuiTexture, TextMetrics};

new_key_type! {
    pub struct GuiNode;
}

pub trait Widget {
    fn as_any(&mut self) -> &mut dyn Any;
    fn draw(&mut self, context: &mut DrawContext, rect: Rect);
    fn handle_input(&mut self, _node: GuiNode, _event_system: &mut GuiEventSystem, _input: GuiInputEvent) -> bool { false }
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
}

impl GuiItem {
    fn new() -> GuiItem {
        GuiItem { rect: Cell::default(), layout: Layout::default() }
    }
    fn with_layout(layout: Layout) -> GuiItem {
        GuiItem { rect: Cell::default(), layout }
    }
}

struct GuiWidgets {
    widgets: SecondaryMap<GuiNode, Box<dyn Widget>>,
}

impl GuiWidgets {
    fn new() -> GuiWidgets {
        GuiWidgets { widgets: SecondaryMap::new() }
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
    fn handle_input(&mut self, node: GuiNode, event_system: &mut GuiEventSystem, input: GuiInputEvent) -> bool {
        if let Some(widget) = self.widgets.get_mut(node) {
            widget.handle_input(node, event_system, input)
        }
        else { false }
    }
    fn handle_cursor_moved(&mut self, forest: &Forest<GuiNode, GuiItem>, node: GuiNode, event_system: &mut GuiEventSystem, cursor_pos: Point) -> bool {
        for child in forest.iter_children(node) {
            if self.handle_cursor_moved(forest, *child, event_system, cursor_pos) {
                return true;
            }
        }
        let rect = forest.get(node).rect.get();
        if rect.contains(cursor_pos) {
            let relative_pos = cursor_pos.relative_to(rect.position);
            self.handle_input(node, event_system, GuiInputEvent::CursorMoved(relative_pos))
        }
        else { false }
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
    fn handle_event(&mut self, widgets: &mut GuiWidgets, event: &GuiActionEvent) {
        match event {
            GuiActionEvent::Hover(node) => self.set_hovered(widgets, *node),
            GuiActionEvent::Focus(_node) => unimplemented!(),
            _ => (),
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
    input_state: GuiInputState,
    render_root: GuiNode,
    event_system: GuiEventSystem,
}

impl Gui {
    pub fn new() -> Gui {
        let mut forest = Forest::new();
        let render_root = forest.add(GuiItem::new());
        Gui {
            forest,
            input_state: GuiInputState::new(),
            widgets: GuiWidgets::new(),
            render_root,
            event_system: GuiEventSystem::new(),
        }
    }

    pub fn root(&self) -> GuiNode {
        self.render_root
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
    pub fn add<W>(&mut self, parent: GuiNode, layout: Layout, widget: W) -> WidgetNode<W> where W: Widget + 'static {
        let node = self.forest.add_child(parent, GuiItem::with_layout(layout));
        self.widgets.insert(node, widget);
        WidgetNode::new(node)
    }
    pub fn set_node_layout(&mut self, node: GuiNode, layout: Layout) {
        let item = self.forest.get_mut(node);
        item.layout = layout;
    }

    pub fn draw(&mut self, context: &mut DrawContext) {
        let root_rect = self.forest.get(self.render_root).rect.get();
        self.widgets.draw_node(&self.forest, self.render_root, context, root_rect);
    }

    pub fn layout_if_needed(&self, parent_size: Size) {
        let root_rect = self.forest.get(self.render_root).rect.get();
        if root_rect.size != parent_size {
            self.layout(self.render_root, Rect { position: Point::origin(), size: parent_size }, None);
        }
    }
    fn layout(&self, node: GuiNode, parent_rect: Rect, previous_sibling_rect: Option<Rect>) -> Rect {
        let item = self.forest.get(node);
        let rect = if node == self.render_root {
            parent_rect
        } else {
            let context = LayoutContext { parent_rect, previous_sibling_rect };
            item.layout.layout_before_children(&context)
        };
        item.rect.set(rect);
        let mut previous_child_rect = None;
        for child in self.iter_children(node) {
            let child_rect = self.layout(*child, rect, previous_child_rect);
            previous_child_rect = Some(child_rect);
        }
        rect
    }

    fn fire_input(&mut self, input: GuiInputEvent) {
        match input {
            GuiInputEvent::CursorMoved(cursor_pos) => {
                if !self.widgets.handle_cursor_moved(&self.forest, self.render_root, &mut self.event_system, cursor_pos) {
                    // If the cursor isn't over anything, set the hovered widget to null.
                    self.input_state.set_hovered(&mut self.widgets, GuiNode::null());
                }
            }
            GuiInputEvent::PrimaryButton(_) => { self.widgets.handle_input(self.input_state.hovered, &mut self.event_system, input); }
        }
    }
    pub fn process_input<A, F>(&mut self, actions: &A, mut handler: F) where A: GuiInputActions, F: FnMut(GuiActionEvent) {
        // Convert input actions to GuiInputEvent and send them to relevant widgets.
        // Widgets respond to GuiInputEvents by sending GuiActionEvents to the event system.
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

        // Process the resulting GuiActionEvents.
        let widgets = &mut self.widgets;
        let input_state = &mut self.input_state;
        self.event_system.dispatch_queue(move |event| {
            input_state.handle_event(widgets, &event);
            handler(event);
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
