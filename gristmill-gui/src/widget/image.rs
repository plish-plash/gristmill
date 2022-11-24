use crate::{
    widget::{StyleQuery, StyleValues, Widget},
    Gui, GuiDraw, GuiFlags, GuiLayout, GuiNode, GuiNodeExt, GuiTexture,
};
use gristmill::{
    geom2d::{Rect, Size},
    Color, Obj,
};

pub struct Image(Obj<GuiNode>);

impl Image {
    const DEFAULT_SIZE: Size = Size {
        width: 64,
        height: 64,
    };

    pub fn set_texture(&self, texture: GuiTexture) {
        self.0.write().draw = GuiDraw::Rect(texture, crate::color::WHITE);
    }
    pub fn set_texture_and_color(&self, texture: GuiTexture, color: Color) {
        self.0.write().draw = GuiDraw::Rect(texture, color);
    }

    pub(crate) fn default_style() -> StyleValues {
        let mut style = StyleValues::new();
        style.set("color", crate::color::WHITE);
        style.set("size", Image::DEFAULT_SIZE);
        style
    }
}

impl Widget for Image {
    fn class_name() -> &'static str {
        "Image"
    }
    fn new(_gui: &mut Gui, parent: Obj<GuiNode>) -> Self {
        let flags = GuiFlags {
            pointer_opaque: true,
            ..Default::default()
        };
        let draw = GuiDraw::Rect(GuiTexture::default(), crate::color::WHITE);
        let node = parent.add_child(GuiNode::new(
            flags,
            draw,
            Rect::from_size(Image::DEFAULT_SIZE),
        ));
        Image(node)
    }
    fn apply_style(&mut self, style: StyleQuery) {
        let mut write_guard = self.0.write();
        write_guard.draw = GuiDraw::Rect(
            GuiTexture::default(),
            style.get("color", crate::color::WHITE),
        );
        write_guard.layout =
            GuiLayout::Child(Rect::from_size(style.get("size", Image::DEFAULT_SIZE)));
    }
    fn node(&self) -> Obj<GuiNode> {
        self.0.clone()
    }
}
