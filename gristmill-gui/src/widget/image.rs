use crate::{
    widget::{StyleValues, Widget, WidgetNode, WidgetNodeExt, WidgetStyle},
    Gui, GuiNode, GuiNodeExt, GuiNodeId, NodeDraw,
};
use gristmill_core::Color;
use gristmill_render::Texture;
use std::any::Any;

pub struct Image(GuiNodeId);

impl Image {
    pub fn set_texture(&self, gui: &mut Gui, texture: Option<Texture>) {
        if let Some(node) = self.node_data(gui) {
            node.draw = NodeDraw::Rect(texture, Color::WHITE);
        }
    }
    pub fn set_texture_and_color(&self, gui: &mut Gui, texture: Option<Texture>, color: Color) {
        if let Some(node) = self.node_data(gui) {
            node.draw = NodeDraw::Rect(texture, color);
        }
    }
}

impl Widget for Image {
    fn class_name() -> &'static str {
        "image"
    }
    fn new(gui: &mut Gui, parent: GuiNodeId, mut style: StyleValues) -> Self {
        let texture = style.widget_value("texture", None);
        let color = style.widget_value("color", Color::WHITE);
        let node = parent.add_child(
            gui,
            GuiNode::new(style.widget_layout(), NodeDraw::Rect(texture, color)),
        );
        Image(node)
    }
}

impl WidgetNode for Image {
    fn as_any_box(self: Box<Self>) -> Box<dyn Any> {
        self
    }
    fn node(&self) -> GuiNodeId {
        self.0
    }
}
