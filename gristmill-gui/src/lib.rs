mod backend;
mod render;
pub mod unpack;
pub mod widget;

use glyph_brush::OwnedSection;
use gristmill::input::CursorAction;
use gristmill::math::IVec2;
use gristmill::{geom2d::*, Color, Obj, Objects};
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::sync::Arc;

use crate::render::GuiTexture;
use crate::widget::{InputState, WidgetBehavior, WidgetObj, WidgetStyles};

pub use backend::GuiRenderer;

mod color {
    use gristmill::color::{rgb::Rgb, Alpha};
    use gristmill::Color;
    use std::marker::PhantomData;
    pub const WHITE: Color = Alpha {
        color: Rgb {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            standard: PhantomData,
        },
        alpha: 1.0,
    };
    pub const BLACK: Color = Alpha {
        color: Rgb {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            standard: PhantomData,
        },
        alpha: 1.0,
    };
}

pub struct GuiFlags {
    pub visible: bool,
    pub pointer_opaque: bool,
}

impl Default for GuiFlags {
    fn default() -> Self {
        GuiFlags {
            visible: true,
            pointer_opaque: false,
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum GuiLayout {
    Child(Rect),
    Fill(EdgeRect),
    Center(Size),
    Row { spacing: i32, x_size: u32 },
    Column { spacing: i32, y_size: u32 },
}

impl Default for GuiLayout {
    fn default() -> Self {
        GuiLayout::Child(Rect::ZERO)
    }
}

impl GuiLayout {
    pub fn fill() -> GuiLayout {
        GuiLayout::Fill(EdgeRect::ZERO)
    }
    fn layout(&self, parent_rect: Rect, previous_sibling: Option<Rect>) -> Rect {
        match self {
            GuiLayout::Child(rect) => *rect + parent_rect.position,
            GuiLayout::Fill(insets) => parent_rect.inset(*insets),
            GuiLayout::Center(size) => {
                let off = IVec2::new((size.width / 2) as i32, (size.height / 2) as i32);
                Rect::new(parent_rect.center() - off, *size)
            }
            GuiLayout::Row { spacing, x_size } => {
                let x = previous_sibling
                    .map(|r| r.top_right().x + *spacing)
                    .unwrap_or(parent_rect.position.x);
                Rect::new(
                    IVec2::new(x, parent_rect.position.y),
                    Size::new(*x_size, parent_rect.size.height),
                )
            }
            GuiLayout::Column { spacing, y_size } => {
                let y = previous_sibling
                    .map(|r| r.bottom_left().y + *spacing)
                    .unwrap_or(parent_rect.position.y);
                Rect::new(
                    IVec2::new(parent_rect.position.x, y),
                    Size::new(parent_rect.size.width, *y_size),
                )
            }
        }
    }
}

#[derive(Clone)]
pub enum GuiDraw {
    None,
    Rect(GuiTexture, Color),
    Text(OwnedSection),
}

impl Default for GuiDraw {
    fn default() -> Self {
        GuiDraw::None
    }
}

#[derive(Default)]
pub struct GuiNode {
    pub flags: GuiFlags,
    pub layout: GuiLayout,
    pub draw: GuiDraw,
    pub offset: Rect,
    rect: Cell<Rect>,
    visible: Cell<bool>,
    children: Vec<Obj<GuiNode>>,
}

impl GuiNode {
    pub fn new(flags: GuiFlags, draw: GuiDraw, rect: Rect) -> GuiNode {
        GuiNode {
            flags,
            layout: GuiLayout::Child(rect),
            draw,
            offset: Rect::ZERO,
            rect: Cell::default(),
            visible: Cell::default(),
            children: Vec::new(),
        }
    }
    pub fn with_layout(layout: GuiLayout) -> GuiNode {
        GuiNode {
            layout,
            ..Default::default()
        }
    }
    pub fn with_draw_and_layout(draw: GuiDraw, layout: GuiLayout) -> GuiNode {
        GuiNode {
            layout,
            draw,
            ..Default::default()
        }
    }

    fn get_draw_rect(&self) -> Rect {
        let mut rect = self.rect.get();
        rect.position += self.offset.position;
        rect.size += self.offset.size;
        rect
    }
}

pub trait GuiNodeExt {
    fn add_child(&self, node: GuiNode) -> Obj<GuiNode>;
    fn remove_child(&self, node: &Obj<GuiNode>) -> bool;
    fn visit_children<F>(&self, f: F)
    where
        F: FnMut(&Self);
    fn visit_descendants<F>(&self, f: &mut F)
    where
        F: FnMut(&Self),
    {
        self.visit_children(|child| {
            f(child);
            child.visit_descendants(f);
        });
    }
}

impl GuiNodeExt for Obj<GuiNode> {
    fn add_child(&self, node: GuiNode) -> Obj<GuiNode> {
        let child = self.objects().insert(node);
        let mut write_guard = self.write();
        write_guard.children.push(child.clone());
        child
    }
    fn remove_child(&self, child: &Obj<GuiNode>) -> bool {
        let mut write_guard = self.write();
        let index = if let Some(index) = write_guard.children.iter().position(|ch| ch == child) {
            index
        } else {
            return false;
        };
        write_guard.children.remove(index);
        true
    }
    fn visit_children<F>(&self, mut f: F)
    where
        F: FnMut(&Self),
    {
        let read_guard = self.read();
        for child in read_guard.children.iter() {
            f(child);
        }
    }
}

pub trait GuiInputActions {
    fn primary(&self) -> &CursorAction;
}

pub struct Gui {
    styles: Arc<WidgetStyles>,
    viewport: Rect,
    nodes: Objects<GuiNode>,
    behaviors: Objects<Box<dyn WidgetBehavior>>,
    root: Obj<GuiNode>,
}

impl Default for Gui {
    fn default() -> Self {
        Gui::new()
    }
}

impl Gui {
    pub fn new() -> Gui {
        Gui::with_styles(WidgetStyles::new())
    }
    pub fn with_styles(styles: WidgetStyles) -> Gui {
        let nodes = Objects::new();
        let root = nodes.insert(GuiNode::default());
        Gui {
            styles: Arc::new(styles),
            viewport: Rect::ZERO,
            nodes,
            behaviors: Objects::new(),
            root,
        }
    }
    pub fn update<I>(&mut self, input: &I)
    where
        I: GuiInputActions,
    {
        self.nodes.cleanup();
        self.behaviors.cleanup();

        // Layout all nodes.
        for (_, node) in self.nodes.read().iter() {
            node.visible.set(false);
        }
        Self::layout_children(&self.root, self.viewport, true);

        // Find the node the cursor is over.
        fn check_cursor_over(cursor: &CursorAction, node: &Obj<GuiNode>) -> Option<Obj<GuiNode>> {
            let node_data = node.read();
            for child in node_data.children.iter().rev() {
                if let Some(pointer_over) = check_cursor_over(cursor, child) {
                    return Some(pointer_over);
                }
            }
            if node_data.visible.get()
                && node_data.flags.pointer_opaque
                && node_data.rect.get().contains(cursor.position())
            {
                Some(node.clone())
            } else {
                None
            }
        }
        let cursor_over = check_cursor_over(input.primary(), &self.root);

        // Update widget behaviors.
        let mut state = InputState {
            input,
            cursor_over: false,
        };
        for (_, behavior) in self.behaviors.write().iter_mut() {
            state.cursor_over = cursor_over == Some(behavior.node());
            behavior.update(state);
        }
    }
    fn layout_children(node: &Obj<GuiNode>, parent_rect: Rect, parent_visible: bool) {
        let mut previous_rect = None;
        node.visit_children(|child| {
            let node_data = child.read();
            let rect = node_data.layout.layout(parent_rect, previous_rect);
            let visible = parent_visible && node_data.flags.visible;
            node_data.rect.set(rect);
            node_data.visible.set(visible);
            Self::layout_children(child, rect, visible);
            previous_rect = Some(rect);
        });
    }

    pub fn styles(&mut self) -> Arc<WidgetStyles> {
        self.styles.clone()
    }

    pub fn root(&self) -> Obj<GuiNode> {
        self.root.clone()
    }

    pub fn register_behavior<B>(&self, behavior: B) -> WidgetObj<B>
    where
        B: WidgetBehavior,
    {
        WidgetObj::new(self.behaviors.insert(Box::new(behavior)))
    }
}
