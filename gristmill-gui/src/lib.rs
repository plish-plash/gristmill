mod render;
pub mod unpack;
pub mod widget;

pub use glyph_brush::{OwnedSection, OwnedText};
use std::rc::{Rc, Weak};

use crate::{
    render::GuiRenderer,
    unpack::Unpacker,
    widget::{Widget, WidgetBehavior, WidgetInput, WidgetStyles},
};
use gristmill_core::{
    asset::{AssetStorage, AssetWriteExt},
    geom2d::*,
    input::InputActions,
    math::IVec2,
    new_storage_types, Color,
};
use gristmill_render::{texture_rect::TextureRectRenderer, RenderContext, Renderable, Texture};
use serde::{Deserialize, Serialize};

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

new_storage_types!(pub type GuiNodeStorage = <GuiNodeId, GuiNode>);

#[derive(Default)]
pub struct GuiNode {
    pub flags: GuiFlags,
    pub layout: GuiLayout,
    pub draw: GuiDraw,
    pub offset: IRect,
    rect: IRect,
    z: u16,
    visible: bool,
    children: Vec<GuiNodeId>,
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
    fn add_child(&self, gui: &mut Gui, child: GuiNode) -> GuiNodeId;
}
impl GuiNodeExt for GuiNodeId {
    fn add_child(&self, gui: &mut Gui, child: GuiNode) -> GuiNodeId {
        let key = gui.nodes.insert(child);
        if let Some(node) = gui.nodes.get_mut(*self) {
            node.children.push(key);
        }
        key
    }
}

pub struct Gui {
    renderer: GuiRenderer,
    textures: AssetStorage<Texture>,
    styles: Rc<WidgetStyles>,
    viewport: IRect,
    nodes: GuiNodeStorage,
    root: GuiNodeId,
    behaviors: Vec<Weak<dyn WidgetBehavior>>,
    unpacker: Unpacker,
}

impl Gui {
    pub fn new(context: &mut RenderContext) -> Self {
        Self::with_styles(
            context,
            WidgetStyles::load_or_save("gui_styles.toml", WidgetStyles::with_all_defaults),
        )
    }
    pub fn with_styles(context: &mut RenderContext, styles: WidgetStyles) -> Self {
        let mut nodes = GuiNodeStorage::default();
        let root = nodes.insert(GuiNode::default());
        Gui {
            renderer: GuiRenderer::new(context),
            textures: AssetStorage::new(),
            styles: Rc::new(styles),
            viewport: IRect::ZERO,
            nodes,
            root,
            behaviors: Vec::new(),
            unpacker: Unpacker::with_standard_widgets(),
        }
    }

    pub fn rect_renderer(&mut self) -> &mut TextureRectRenderer {
        self.renderer.rect_renderer()
    }
    pub fn load_textures(&mut self, context: &mut RenderContext) {
        self.styles.load_textures(context, &mut self.textures);
    }

    pub fn update(&mut self, input: &InputActions) {
        // Layout all nodes.
        fn layout_children(
            nodes: &mut GuiNodeStorage,
            node: GuiNodeId,
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
                layout_children(nodes, child, rect, visible, z + 1);
                previous_rect = Some(rect);
            }
        }

        for (_, node) in self.nodes.iter_mut() {
            node.visible = false;
        }
        let root_z = {
            let root = self.nodes.get_mut(self.root).unwrap();
            root.visible = true;
            root.z
        };
        layout_children(&mut self.nodes, self.root, self.viewport, true, root_z + 1);

        // Find the node the pointer is over.
        fn check_pointer_over(
            nodes: &GuiNodeStorage,
            node: GuiNodeId,
            pointer: IVec2,
        ) -> Option<GuiNodeId> {
            let node_data = nodes.get(node).unwrap();
            if !node_data.visible {
                return None;
            }
            for child in node_data.children.iter().rev() {
                if let Some(pointer_over) = check_pointer_over(nodes, *child, pointer) {
                    return Some(pointer_over);
                }
            }
            if node_data.flags.pointer_opaque && node_data.rect.contains(pointer) {
                Some(node)
            } else {
                None
            }
        }

        let pointer_state = input.get("primary");
        let pointer_over = pointer_state
            .pointer()
            .and_then(|p| check_pointer_over(&self.nodes, self.root, p.as_ivec2()));

        // Update widget behaviors.
        let input = WidgetInput {
            state: pointer_state,
            pointer_over,
        };
        self.behaviors.retain_mut(|behavior| {
            if let Some(behavior) = behavior.upgrade() {
                behavior.update(&mut self.nodes, &input);
                true
            } else {
                false
            }
        });
    }

    pub fn nodes(&self) -> &GuiNodeStorage {
        &self.nodes
    }
    pub fn nodes_mut(&mut self) -> &mut GuiNodeStorage {
        &mut self.nodes
    }

    pub fn root(&self) -> GuiNodeId {
        self.root
    }
    pub fn set_root_z(&mut self, z: u16) {
        if let Some(node) = self.nodes.get_mut(self.root) {
            node.z = z;
        }
    }

    pub fn register_behavior<B: WidgetBehavior>(&mut self, behavior: B) -> Rc<B> {
        let behavior = Rc::new(behavior);
        let dyn_behavior: Rc<dyn WidgetBehavior> = behavior.clone();
        self.behaviors.push(Rc::downgrade(&dyn_behavior));
        behavior
    }

    pub fn create_widget<W>(&mut self, parent: GuiNodeId, class: Option<&str>) -> W
    where
        W: Widget + 'static,
    {
        let mut classes = vec![W::type_name()];
        if let Some(class) = class {
            classes.push(class);
        }
        let styles = self.styles.clone();
        let style = styles.query(classes);
        W::new(self, parent, style)
    }
}

impl Renderable for Gui {
    fn pre_render(&mut self, context: &mut RenderContext) {
        self.viewport = context.viewport().as_irect();
        self.renderer.process(context, &self.nodes);
    }
    fn render(&mut self, context: &mut RenderContext) {
        self.renderer.draw_all(context);
    }
}
