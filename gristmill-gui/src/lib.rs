mod render;
pub mod unpack;
pub mod widget;

use crate::widget::{Widget, WidgetBehavior, WidgetInput, WidgetStyles};
use glyph_brush::OwnedSection;
use gristmill::object::DenseSlotMap;
use gristmill::{
    geom2d::*, input::InputActions, math::IVec2, new_object_type, object::ObjectCollection,
    render::texture::Texture, Color,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub use render::GuiRenderer;

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
    Rect(Option<Texture>, Color),
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
    rect: Rect,
    z: u16,
    visible: bool,
    children: Vec<GuiNodeKey>,
}

impl GuiNode {
    pub fn new(flags: GuiFlags, draw: GuiDraw, rect: Rect) -> GuiNode {
        GuiNode {
            flags,
            layout: GuiLayout::Child(rect),
            draw,
            offset: Rect::ZERO,
            rect: Rect::ZERO,
            z: 0,
            visible: false,
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

    fn draw_rect(&self) -> (Rect, u16) {
        let mut rect = self.rect;
        rect.position += self.offset.position;
        rect.size += self.offset.size;
        (rect, self.z)
    }
}

pub trait GuiNodeExt {
    fn add_child(&self, node: GuiNode) -> GuiNodeObj;
    fn remove_child(&self, node: &GuiNodeObj) -> bool;
}

impl GuiNodeExt for GuiNodeObj {
    fn add_child(&self, node: GuiNode) -> GuiNodeObj {
        let child = self.objects().insert(node);
        let mut write_guard = self.write();
        write_guard.children.push(child.key());
        child
    }
    fn remove_child(&self, child: &GuiNodeObj) -> bool {
        let mut write_guard = self.write();
        let index = if let Some(index) = write_guard
            .children
            .iter()
            .position(|ch| *ch == child.key())
        {
            index
        } else {
            return false;
        };
        write_guard.children.remove(index);
        true
    }
}

new_object_type!(GuiNode, GuiNodeKey, GuiNodeObj, GuiNodeObjects);

pub struct Gui {
    styles: Arc<WidgetStyles>,
    viewport: Rect,
    nodes: GuiNodeObjects,
    behaviors: Vec<Arc<dyn WidgetBehavior>>,
    root: GuiNodeKey,
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
        let nodes = GuiNodeObjects::default();
        let root = nodes.insert(GuiNode::default()).key();
        Gui {
            styles: Arc::new(styles),
            viewport: Rect::ZERO,
            nodes,
            behaviors: Vec::new(),
            root,
        }
    }
    pub fn update(&mut self, input: &InputActions) {
        // Layout all nodes.
        let mut write_guard = self.nodes.write().unwrap();
        for (_, node) in write_guard.iter_mut() {
            node.visible = false;
        }
        let root_z = write_guard.get(self.root).unwrap().z;
        Self::layout_children(&mut write_guard, self.root, self.viewport, true, root_z + 1);

        // Find the node the pointer is over.
        fn check_pointer_over(
            nodes: &DenseSlotMap<GuiNodeKey, GuiNode>,
            node: GuiNodeKey,
            pointer: IVec2,
        ) -> Option<GuiNodeKey> {
            let node_data = &nodes[node];
            for child in node_data.children.iter().rev() {
                if let Some(pointer_over) = check_pointer_over(nodes, *child, pointer) {
                    return Some(pointer_over);
                }
            }
            if node_data.visible
                && node_data.flags.pointer_opaque
                && node_data.rect.contains(pointer)
            {
                Some(node)
            } else {
                None
            }
        }
        let pointer_state = input.get("primary");
        let pointer_over = pointer_state
            .pointer()
            .and_then(|p| check_pointer_over(&write_guard, self.root, p.as_ivec2()));
        drop(write_guard);

        // Update widget behaviors.
        let input = WidgetInput {
            state: pointer_state,
            pointer_over,
        };
        for behavior in self.behaviors.iter() {
            behavior.update(input);
        }
    }
    fn layout_children(
        nodes: &mut DenseSlotMap<GuiNodeKey, GuiNode>,
        node: GuiNodeKey,
        parent_rect: Rect,
        parent_visible: bool,
        z: u16,
    ) {
        let mut previous_rect = None;
        let children = nodes.get(node).unwrap().children.clone();
        for child in children {
            let node_data = nodes.get_mut(child).unwrap();
            let visible = parent_visible && node_data.flags.visible;
            let rect = node_data.layout.layout(parent_rect, previous_rect);
            node_data.visible = visible;
            node_data.rect = rect;
            node_data.z = z;
            Self::layout_children(nodes, child, rect, visible, z + 1);
            previous_rect = Some(rect);
        }
    }

    pub fn styles(&mut self) -> Arc<WidgetStyles> {
        self.styles.clone()
    }

    pub fn root(&self) -> GuiNodeObj {
        GuiNodeObj::from_key(self.nodes.clone(), self.root)
    }
    pub fn set_root_z(&self, z: u16) {
        self.root().write().z = z;
    }

    pub fn register_behavior<B: WidgetBehavior + 'static>(&mut self, behavior: Arc<B>) {
        self.behaviors.push(behavior);
    }

    pub fn create_widget<W: Widget>(&mut self, parent: GuiNodeObj) -> W {
        let mut widget = W::new(self, parent);
        widget.apply_style(self.styles.query([W::class_name()]));
        widget
    }
}
