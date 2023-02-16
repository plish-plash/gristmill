pub mod layout;
mod render;
pub mod unpack;
pub mod widget;

pub use glyph_brush::{OwnedSection, OwnedText};
use layout::GuiLayout;
use std::{
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::{
    render::GuiRenderer,
    unpack::Unpacker,
    widget::{Widget, WidgetBehavior, WidgetInput, WidgetStyles},
};
use gristmill_core::{
    asset::AssetResult, geom2d::*, input::InputActions, math::IVec2, new_storage_types,
    slotmap::SecondaryMap, Color,
};
use gristmill_render::{texture_rect::TextureRectRenderer, RenderContext, Renderable, Texture};

pub struct NodeFlags {
    pub visible: bool,
    pub pointer_opaque: bool,
}

impl Default for NodeFlags {
    fn default() -> Self {
        NodeFlags {
            visible: true,
            pointer_opaque: false,
        }
    }
}

#[derive(Default)]
pub enum Anchor {
    #[default]
    Begin,
    Middle,
    End,
}

impl std::str::FromStr for Anchor {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "begin" | "Begin" | "left" | "Left" | "top" | "Top" => Ok(Anchor::Begin),
            "middle" | "Middle" | "center" | "Center" => Ok(Anchor::Middle),
            "end" | "End" | "right" | "Right" | "bottom" | "Bottom" => Ok(Anchor::End),
            _ => Err(()),
        }
    }
}

#[derive(Default)]
pub struct NodeLayout {
    pub size: IVec2,
    pub margin: EdgeRect,
    pub anchors: (Anchor, Anchor),
    pub child_layout: String,
    pub child_spacing: i32,
}

impl NodeLayout {
    pub fn width(&self) -> i32 {
        self.size.x + self.margin.left + self.margin.right
    }
    pub fn height(&self) -> i32 {
        self.size.y + self.margin.top + self.margin.bottom
    }
    pub fn horizontal(&self, container_x: i32, container_width: i32) -> (i32, i32) {
        if self.size.x == 0 {
            (container_x, container_width)
        } else {
            let width = self.width();
            let x = container_x
                + match self.anchors.0 {
                    Anchor::Begin => 0,
                    Anchor::Middle => (container_width / 2) - (width / 2),
                    Anchor::End => container_width - width,
                };
            (x, width)
        }
    }
    pub fn vertical(&self, container_y: i32, container_height: i32) -> (i32, i32) {
        if self.size.y == 0 {
            (container_y, container_height)
        } else {
            let height = self.height();
            let y = container_y
                + match self.anchors.1 {
                    Anchor::Begin => 0,
                    Anchor::Middle => (container_height / 2) - (height / 2),
                    Anchor::End => container_height - height,
                };
            (y, height)
        }
    }
}

#[derive(Clone)]
pub enum NodeDraw {
    None,
    Rect(Option<Texture>, Color),
    Text(OwnedSection),
}

impl Default for NodeDraw {
    fn default() -> Self {
        NodeDraw::None
    }
}

new_storage_types!(pub type GuiNodeStorage = <GuiNodeId, GuiNode>);

#[derive(Default)]
pub struct GuiNode {
    pub flags: NodeFlags,
    pub layout: NodeLayout,
    pub draw: NodeDraw,
    pub offset: IRect,
    visible: bool,
    rect: IRect,
    z: u16,
}

impl GuiNode {
    pub fn new(layout: NodeLayout, draw: NodeDraw) -> GuiNode {
        GuiNode {
            layout,
            draw,
            ..Default::default()
        }
    }
    pub fn with_draw(draw: NodeDraw) -> GuiNode {
        GuiNode {
            draw,
            ..Default::default()
        }
    }

    fn draw_rect(&self) -> (IRect, u16) {
        (self.rect.add_components(self.offset), self.z)
    }
}

pub trait GuiNodeExt {
    fn add_child(&self, gui: &mut Gui, child: GuiNode) -> GuiNodeId;
}
impl GuiNodeExt for GuiNodeId {
    fn add_child(&self, gui: &mut Gui, child: GuiNode) -> GuiNodeId {
        let children = gui
            .node_children
            .entry(*self)
            .expect("node has been removed")
            .or_default();
        let key = gui.nodes.insert(child);
        children.push(key);
        key
    }
}

pub struct Gui {
    renderer: GuiRenderer,
    styles: WidgetStyles,
    layouts: HashMap<String, Box<dyn GuiLayout>>,
    default_layout: Box<dyn GuiLayout>,

    nodes: GuiNodeStorage,
    node_children: SecondaryMap<GuiNodeId, Vec<GuiNodeId>>,
    root: GuiNodeId,
    behaviors: Vec<Weak<dyn WidgetBehavior>>,
    unpacker: Unpacker,
}

impl Gui {
    fn default_layouts() -> HashMap<String, Box<dyn GuiLayout>> {
        let mut layouts: HashMap<String, Box<dyn GuiLayout>> = HashMap::new();
        layouts.insert("hbox".to_owned(), Box::<layout::HBox>::default());
        layouts.insert("vbox".to_owned(), Box::<layout::VBox>::default());
        layouts
    }

    pub fn new(context: &mut RenderContext, styles: WidgetStyles) -> Self {
        let mut nodes = GuiNodeStorage::default();
        let root = nodes.insert(GuiNode {
            rect: context.viewport().as_irect(),
            ..Default::default()
        });
        Gui {
            renderer: GuiRenderer::new(context),
            styles,
            layouts: Self::default_layouts(),
            default_layout: Box::<layout::Anchor>::default(),
            nodes,
            node_children: SecondaryMap::new(),
            root,
            behaviors: Vec::new(),
            unpacker: Unpacker::with_standard_widgets(),
        }
    }
    pub fn load_styles(context: &mut RenderContext) -> AssetResult<Self> {
        let styles = WidgetStyles::load_asset(context)?;
        Ok(Self::new(context, styles))
    }

    pub fn rect_renderer(&mut self) -> &mut TextureRectRenderer {
        self.renderer.rect_renderer()
    }
    pub fn styles(&self) -> &WidgetStyles {
        &self.styles
    }

    fn layout(&mut self, node: GuiNodeId) {
        let node_data = if let Some(data) = self.nodes.get(node) {
            data
        } else {
            return;
        };
        if !node_data.visible {
            return;
        }
        let node_rect = node_data.rect;
        let mut z = node_data.z;
        let children = if let Some(children) = self.node_children.get_mut(node) {
            children
        } else {
            return;
        };
        let child_layout = self
            .layouts
            .get_mut(&node_data.layout.child_layout)
            .unwrap_or(&mut self.default_layout);
        child_layout.begin_layout(node_rect, node_data.layout.child_spacing);
        children.retain_mut(|child| {
            let child_data = if let Some(data) = self.nodes.get_mut(*child) {
                data
            } else {
                return false;
            };
            child_data.visible = child_data.flags.visible;
            let rect = child_layout.layout_child(&child_data.layout);
            child_data.rect = rect.inset(child_data.layout.margin);
            z += 1;
            child_data.z = z;
            true
        });
        for child in children.clone() {
            self.layout(child);
        }
    }
    fn find_pointer_over(&self, node: GuiNodeId, pointer: IVec2) -> Option<GuiNodeId> {
        let node_data = self.nodes.get(node)?;
        if !node_data.visible {
            return None;
        }
        if let Some(children) = self.node_children.get(node) {
            for child in children.iter().rev() {
                if let Some(pointer_over) = self.find_pointer_over(*child, pointer) {
                    return Some(pointer_over);
                }
            }
        }
        if node_data.flags.pointer_opaque && node_data.rect.contains(pointer) {
            Some(node)
        } else {
            None
        }
    }

    pub fn update(&mut self, input: &InputActions) {
        // Layout all nodes.
        for node in self.nodes.values_mut() {
            node.visible = false;
        }
        self.nodes
            .get_mut(self.root)
            .expect("root node has been removed")
            .visible = true;
        self.layout(self.root);

        // Find the node the pointer is over.
        let pointer_state = input.get("primary");
        let pointer_over = pointer_state
            .pointer()
            .and_then(|p| self.find_pointer_over(self.root, p.as_ivec2()));

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
        if let Some(root_node) = self.nodes.get_mut(self.root) {
            root_node.z = z;
        }
    }

    pub fn register_behavior<B: WidgetBehavior>(&mut self, behavior: B) -> Rc<B> {
        let behavior = Rc::new(behavior);
        let dyn_behavior: Rc<dyn WidgetBehavior> = behavior.clone();
        self.behaviors.push(Rc::downgrade(&dyn_behavior));
        behavior
    }

    pub fn create_widget<W: Widget>(&mut self, parent: GuiNodeId) -> W {
        let style = self.styles.query(std::iter::once(W::class_name()));
        W::new(self, parent, style)
    }
}

impl Renderable for Gui {
    fn pre_render(&mut self, context: &mut RenderContext) {
        if let Some(root_node) = self.nodes.get_mut(self.root) {
            root_node.rect = context.viewport().as_irect();
        }
        self.renderer.process(context, &self.nodes);
    }
    fn render(&mut self, context: &mut RenderContext) {
        self.renderer.draw_all(context);
    }
}
