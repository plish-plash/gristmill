pub mod geometry;
pub mod style;
pub mod font;
pub mod signal;
pub mod text;
pub mod color_rect;
pub mod button;
pub mod toggle_group;
pub mod inflate;

use std::{any::Any, collections::HashMap, sync::{Arc, RwLock}};
use stretch::{Stretch, style::Dimension};
use geometry::*;
use signal::Signal;
use crate::color::Color;
use crate::input::{InputActions, CursorAction};
pub use stretch::{node::Node, style::Style};
pub use super::renderer::subpass::gui::{DrawCommand, DrawContext, SizedDrawable, TextDrawable};

pub struct Gui {
    layout: Stretch,
    root_node: Node,
    widget_map: HashMap<Node, Box<dyn Widget>>,
    modification_queue: Arc<GuiModifierQueue>,
}

// TODO need better error reporting for Nodes that no longer exist
impl Gui {
    fn make_root_node(layout: &mut Stretch) -> Node {
        layout.new_node(Style {
            size: stretch::geometry::Size { width: Dimension::Percent(1.), height: Dimension::Percent(1.) },
            ..Default::default()
        }, &[]).unwrap()
    }
    pub fn new() -> Gui {
        let mut layout = Stretch::new();
        let root_node = Gui::make_root_node(&mut layout);
        Gui {
            layout,
            root_node,
            widget_map: HashMap::new(),
            modification_queue: Arc::new(GuiModifierQueue::new()),
        }
    }

    pub fn modification_queue(&self) -> Arc<GuiModifierQueue> {
        self.modification_queue.clone()
    }

    pub fn root_node(&self) -> Node {
        self.root_node
    }
    pub fn remove_all(&mut self) {
        self.layout.clear();
        self.widget_map.clear();
        self.root_node = Gui::make_root_node(&mut self.layout);
    }
    pub fn add_child(&mut self, parent: Node) -> Node {
        let child = self.layout.new_node(Style::default(), &[]).unwrap();
        self.layout.add_child(parent, child).unwrap();
        child
    }
    pub fn add_child_widget<T>(&mut self, parent: Node, widget: T) -> Node where T: WidgetBuilder {
        widget.build(self, parent)
    }
    pub fn set_style(&mut self, node: Node, style: Style) {
        self.layout.set_style(node, style).unwrap();
    }
    pub fn set_style_fill_parent(&mut self, node: Node) {
        self.layout.set_style(node, Style {
            flex_grow: 1.0,
            ..Default::default()
        }).unwrap();
    }

    pub fn widget_mut<T>(&mut self, node: Node) -> Option<&mut T> where T: Widget + 'static {
        self.widget_map.get_mut(&node).and_then(|widget| widget.as_any().downcast_mut())
    }
    pub fn children(&self, node: Node) -> Option<Vec<Node>> {
        self.layout.children(node).ok()
    }
    fn node_rect(&self, node: Node) -> Option<Rect> {
        self.layout.layout(node).ok().cloned().map(From::from)
    }

    pub fn refresh_layout(&mut self, screen_size: Size) {
        // TODO verify that this does nothing if no nodes have changed
        self.layout.compute_layout(self.root_node(), screen_size.into()).unwrap();
    }
    pub fn refresh_drawables(&mut self, context: &mut DrawContext) {
        // TODO also let widgets know if they're currently visible, so they only acquire Drawables when they're going to be rendered.
        for (node, widget) in self.widget_map.iter_mut() {
            widget.refresh_drawables(context);
            // TODO only update the style if the minimum size changes
            if let Some(minimum_size) = widget.minimum_size() {
                let mut style = self.layout.style(*node).unwrap().clone();
                style.min_size = minimum_size.into();
                self.layout.set_style(*node, style).unwrap();
            }
        }
    }
    pub fn draw_widget(&self, parent_position: Point, node: Node) -> (Point, Option<DrawCommand>) {
        let mut rect = self.node_rect(node).unwrap();
        rect.position.offset(parent_position);
        (rect.position, self.widget_map.get(&node).and_then(|widget| {
            widget.draw(rect)
        }))
    }
}

pub trait Widget {
    // TODO use custom downcasting instead of Any.
    fn as_any(&mut self) -> &mut dyn Any;
    // TODO rename to update and add a visible bool parameter.
    fn refresh_drawables(&mut self, context: &mut DrawContext);
    fn minimum_size(&self) -> Option<Size> { None }
    fn draw(&self, rect: Rect) -> Option<DrawCommand>;
    // TODO returning a modifier object is kind of an odd way to update the gui, but it might be fine.
    fn cursor_input(&mut self, _input: &CursorAction, _cursor_over: bool) {}
}

pub trait WidgetBuilder: Sized {
    type Widget: Widget + 'static;
    fn build_widget(self, gui: &mut Gui, node: Node) -> Self::Widget;
    fn build(self, gui: &mut Gui, parent: Node) -> Node {
        let node = gui.add_child(parent);
        let widget = Box::new(self.build_widget(gui, node));
        gui.widget_map.insert(node, widget);
        node
    }
}

impl<T> WidgetBuilder for T where T: Widget + 'static {
    type Widget = T;
    fn build_widget(self, _gui: &mut Gui, _node: Node) -> Self::Widget { self }
}

pub trait GuiModifier {
    fn modify(&self, gui: &mut Gui);
}

pub struct GuiModifierQueue {
    // TODO if write() is called while inside a write lock, the program hangs (it should panic instead).
    // might want to swap for a RefCell to make this more idiomatic.
    queued_modifiers: RwLock<Vec<Box<dyn GuiModifier>>>
}

impl GuiModifierQueue {
    fn new() -> GuiModifierQueue {
        GuiModifierQueue { queued_modifiers: RwLock::default() }
    }
    pub fn enqueue<T>(&self, modification: T) where T: GuiModifier + 'static {
        self.queued_modifiers.write().unwrap().push(Box::new(modification));
    }
}

// ------------------------------------------------------------------------------------------------

pub trait GuiInputActions : InputActions {
    fn gui_primary(&self) -> &CursorAction;
}

impl Gui {
    fn do_modifications(&mut self, modification_queue: Arc<GuiModifierQueue>) {
        let queued_modifiers = {
            modification_queue.queued_modifiers.write().unwrap().split_off(0)
        };
        for modifier in queued_modifiers {
            modifier.modify(self);
        }
    }
    pub fn process_input<I>(&mut self, input: &I) where I: GuiInputActions {
        let mut visitor = CursorInputVisitor(input.gui_primary());
        visitor.walk(self, self.root_node(), Point::default());
        self.do_modifications(self.modification_queue());
    }
}

struct CursorInputVisitor<'a>(&'a CursorAction);

impl<'a> CursorInputVisitor<'a> {
    fn visit(&mut self, gui: &mut Gui, node: Node, parent_position: Point) -> Point {
        let mut rect = gui.node_rect(node).unwrap();
        rect.position.offset(parent_position);
        if let Some(widget) = gui.widget_map.get_mut(&node) {
            let cursor_over = rect.contains(self.0.position());
            widget.cursor_input(self.0, cursor_over);
        }
        rect.position
    }
    fn walk(&mut self, gui: &mut Gui, node: Node, parent_position: Point) {
        let node_position = self.visit(gui, node, parent_position);
        for child in gui.children(node).unwrap() {
            self.walk(gui, child, node_position);
        }
    }
}
