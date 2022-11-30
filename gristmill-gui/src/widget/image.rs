use crate::widget::StyleValue;
use crate::{
    widget::{StyleQuery, StyleValues, Widget},
    Gui, GuiDraw, GuiFlags, GuiLayout, GuiNode, GuiNodeExt, GuiNodeObj,
};
use gristmill::{
    color::Pixel,
    geom2d::{IRect, Size},
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
        style.insert(
            "texture".to_owned(),
            crate::widget::style::make_empty_texture(),
        );
        style.insert(
            "color".to_owned(),
            StyleValue::try_from(crate::color::WHITE.into_raw::<[f32; 4]>()).unwrap(),
        );
        style.insert(
            "size".to_owned(),
            StyleValue::try_from(Image::DEFAULT_SIZE).unwrap(),
        );
        style
    }
}

impl Widget for Image {
    fn class_name() -> &'static str {
        "image"
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
            IRect::from_size(Image::DEFAULT_SIZE),
        ));
        Image(node)
    }
    fn apply_style(&mut self, style: StyleQuery) {
        let mut write_guard = self.0.write();
        let texture = style.get_texture("texture");
        write_guard.draw =
            GuiDraw::Rect(texture, style.get("color").unwrap_or(crate::color::WHITE));
        write_guard.layout = GuiLayout::Child(IRect::from_size(
            style.get("size").unwrap_or(Image::DEFAULT_SIZE),
        ));
    }
    fn node(&self) -> &GuiNodeObj {
        &self.0
    }
}
