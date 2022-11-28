use crate::{
    widget::{StyleQuery, StyleValues, Widget},
    Gui, GuiDraw, GuiFlags, GuiLayout, GuiNode, GuiNodeExt, GuiNodeObj,
};
use gristmill::{
    geom2d::{Rect, Size},
    render::texture::Texture,
    Color,
};

pub struct Image(GuiNodeObj);

impl Image {
    const DEFAULT_SIZE: Size = Size {
        width: 64,
        height: 64,
    };

    pub fn set_texture(&self, texture: Option<Texture>) {
        self.0.write().draw = GuiDraw::Rect(texture, crate::color::WHITE);
    }
    pub fn set_texture_and_color(&self, texture: Option<Texture>, color: Color) {
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
    fn new(_gui: &mut Gui, parent: GuiNodeObj) -> Self {
        let flags = GuiFlags {
            pointer_opaque: true,
            ..Default::default()
        };
        let draw = GuiDraw::Rect(None, crate::color::WHITE);
        let node = parent.add_child(GuiNode::new(
            flags,
            draw,
            Rect::from_size(Image::DEFAULT_SIZE),
        ));
        Image(node)
    }
    fn apply_style(&mut self, style: StyleQuery) {
        let mut write_guard = self.0.write();
        write_guard.draw = GuiDraw::Rect(None, style.get("color", crate::color::WHITE));
        write_guard.layout =
            GuiLayout::Child(Rect::from_size(style.get("size", Image::DEFAULT_SIZE)));
    }
    fn node(&self) -> &GuiNodeObj {
        &self.0
    }
}
