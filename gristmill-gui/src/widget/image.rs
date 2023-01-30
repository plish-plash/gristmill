use crate::{
    widget::{StyleQuery, StyleValue, StyleValues, Widget, WidgetNode},
    Gui, GuiDraw, GuiFlags, GuiNode, GuiNodeExt, GuiNodeId,
};
use gristmill_core::{
    geom2d::{IRect, Size},
    Color,
};
use gristmill_render::Texture;
use std::any::Any;

pub struct Image(GuiNodeId);

impl Image {
    const DEFAULT_SIZE: Size = Size {
        width: 64,
        height: 64,
    };

    pub fn set_texture(&self, gui: &mut Gui, texture: Option<Texture>) {
        if let Some(node) = self.node_data(gui) {
            node.draw = GuiDraw::Rect(texture, Color::WHITE);
        }
    }
    pub fn set_texture_and_color(&self, gui: &mut Gui, texture: Option<Texture>, color: Color) {
        if let Some(node) = self.node_data(gui) {
            node.draw = GuiDraw::Rect(texture, color);
        }
    }

    pub(crate) fn default_style() -> StyleValues {
        let mut style = StyleValues::new();
        style.insert(
            "texture".to_owned(),
            crate::widget::style::make_empty_texture(),
        );
        style.insert(
            "color".to_owned(),
            StyleValue::try_from(<[f32; 4]>::from(Color::WHITE)).unwrap(),
        );
        style.insert(
            "size".to_owned(),
            StyleValue::try_from(Image::DEFAULT_SIZE).unwrap(),
        );
        style
    }
}

impl Widget for Image {
    fn type_name() -> &'static str {
        "Image"
    }
    fn new(gui: &mut Gui, parent: GuiNodeId, style: StyleQuery) -> Self {
        let texture = style.get_texture(gui, "texture");
        let color = style.get("color").unwrap_or(Color::WHITE);
        let node = parent.add_child(
            gui,
            GuiNode::new(
                GuiFlags::default(),
                GuiDraw::Rect(texture, color),
                IRect::from_size(style.get("size").unwrap_or(Image::DEFAULT_SIZE)),
            ),
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
