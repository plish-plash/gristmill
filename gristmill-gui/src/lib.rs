mod console;
mod render;
pub mod unpack;
pub mod widget;

pub use console::run_game_with_console;
pub use render::GuiRenderer;

use crate::widget::{Widget, WidgetInput, WidgetState, WidgetStyles};
use glyph_brush::OwnedSection;
use gristmill::{
    asset::AssetStorage,
    geom2d::*,
    input::InputActions,
    math::IVec2,
    new_object_type,
    object::{DenseSlotMap, ObjectCollection},
    render::{texture::Texture, RenderContext},
    Color,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

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
    Child(IRect),
    Fill(EdgeRect),
    Center(Size),
    Row { spacing: i32, x_size: u32 },
    Column { spacing: i32, y_size: u32 },
}

impl Default for GuiLayout {
    fn default() -> Self {
        GuiLayout::Child(IRect::ZERO)
    }
}

impl GuiLayout {
    pub fn fill() -> GuiLayout {
        GuiLayout::Fill(EdgeRect::ZERO)
    }
    fn layout(&self, parent_rect: IRect, previous_sibling: Option<IRect>) -> IRect {
        match self {
            GuiLayout::Child(rect) => *rect + parent_rect.position,
            GuiLayout::Fill(insets) => parent_rect.inset(*insets),
            GuiLayout::Center(size) => {
                let off = IVec2::new((size.width / 2) as i32, (size.height / 2) as i32);
                IRect::new(parent_rect.center() - off, *size)
            }
            GuiLayout::Row { spacing, x_size } => {
                let x = previous_sibling
                    .map(|r| r.top_right().x + *spacing)
                    .unwrap_or(parent_rect.position.x);
                IRect::new(
                    IVec2::new(x, parent_rect.position.y),
                    Size::new(*x_size, parent_rect.size.height),
                )
            }
            GuiLayout::Column { spacing, y_size } => {
                let y = previous_sibling
                    .map(|r| r.bottom_left().y + *spacing)
                    .unwrap_or(parent_rect.position.y);
                IRect::new(
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
    pub offset: IRect,
    rect: IRect,
    z: u16,
    visible: bool,
    children: Vec<GuiNodeKey>,
}

impl GuiNode {
    pub fn new(flags: GuiFlags, draw: GuiDraw, rect: IRect) -> GuiNode {
        GuiNode {
            flags,
            layout: GuiLayout::Child(rect),
            draw,
            offset: IRect::ZERO,
            rect: IRect::ZERO,
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

    fn draw_rect(&self) -> (IRect, u16) {
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
    styles: WidgetStyles,
    viewport: IRect,
    nodes: GuiNodeObjects,
    widget_states: Vec<Arc<RwLock<dyn WidgetState>>>,
    root: GuiNodeKey,
}

impl Default for Gui {
    fn default() -> Self {
        let styles = AssetStorage::config()
            .load_or_save_default("gui_styles.toml", WidgetStyles::with_all_defaults)
            .unwrap_or_default();
        Self::with_styles(styles)
    }
}

impl Gui {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn with_styles(styles: WidgetStyles) -> Gui {
        let nodes = GuiNodeObjects::default();
        let root = nodes.insert(GuiNode::default()).key();
        Gui {
            styles,
            viewport: IRect::ZERO,
            nodes,
            widget_states: Vec::new(),
            root,
        }
    }

    pub fn load_textures(&self, context: &mut RenderContext) {
        self.styles.load_textures(context);
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
            .and_then(|a| a.pointer())
            .and_then(|p| check_pointer_over(&write_guard, self.root, p.as_ivec2()));
        drop(write_guard);

        // Update widget states.
        if let Some(input_state) = pointer_state {
            let input = WidgetInput {
                state: input_state,
                pointer_over,
            };
            for state in self.widget_states.iter() {
                state.write().unwrap().update(input);
            }
        }
    }
    fn layout_children(
        nodes: &mut DenseSlotMap<GuiNodeKey, GuiNode>,
        node: GuiNodeKey,
        parent_rect: IRect,
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

    pub fn root(&self) -> GuiNodeObj {
        GuiNodeObj::from_key(self.nodes.clone(), self.root)
    }
    pub fn set_root_z(&self, z: u16) {
        self.root().write().z = z;
    }

    pub fn register_widget_state<S: WidgetState>(&mut self, state: S) -> Arc<RwLock<S>> {
        let arc = Arc::new(RwLock::new(state));
        self.widget_states.push(arc.clone());
        arc
    }
    pub fn create_widget<W: Widget>(&mut self, parent: GuiNodeObj, class: Option<&str>) -> W {
        let mut widget = W::new(self, parent);
        let mut classes = vec![W::class_name()];
        if let Some(class) = class {
            classes.insert(0, class);
        }
        widget.apply_style(self.styles.query(classes));
        widget
    }
}
