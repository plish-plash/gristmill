use crate::widget::{StyleValue, StyleValues, Widget, WidgetType};
use crate::{Gui, GuiDraw, GuiFlags, GuiNode, GuiNodeExt, GuiTexture};
use gristmill::geom2d::{Rect, Size};
use gristmill::{Color, Obj};

pub struct ImageStyle {
    pub draw: GuiDraw,
    pub size: Size,
}

impl ImageStyle {
    fn from_style(style: &StyleValues) -> ImageStyle {
        ImageStyle {
            draw: GuiDraw::Rect(
                GuiTexture::default(),
                style
                    .get("color")
                    .and_then(StyleValue::to_color)
                    .unwrap_or(crate::color::WHITE),
            ),
            size: style
                .get("size")
                .and_then(StyleValue::to_size)
                .unwrap_or(Size::new(64, 64)),
        }
    }
}
impl Default for ImageStyle {
    fn default() -> ImageStyle {
        ImageStyle {
            draw: GuiDraw::Rect(GuiTexture::default(), crate::color::WHITE),
            size: Size::new(64, 64),
        }
    }
}

pub struct Image(Obj<GuiNode>);

impl Image {
    pub fn create_with_image_style(parent: Obj<GuiNode>, style: ImageStyle) -> Image {
        let flags = GuiFlags {
            pointer_opaque: true,
            ..Default::default()
        };
        let node = parent.add_child(GuiNode::new(flags, style.draw, Rect::from_size(style.size)));
        Image(node)
    }

    pub fn set_texture(&self, texture: GuiTexture) {
        self.0.write().draw = GuiDraw::Rect(texture, crate::color::WHITE);
    }
    pub fn set_texture_and_color(&self, texture: GuiTexture, color: Color) {
        self.0.write().draw = GuiDraw::Rect(texture, color);
    }
}

impl Widget for Image {
    fn widget_type() -> WidgetType {
        WidgetType::image()
    }
    fn create_with_style(_gui: &mut Gui, parent: Obj<GuiNode>, style: &StyleValues) -> Image {
        Self::create_with_image_style(parent, ImageStyle::from_style(style))
    }
    fn node(&self) -> Obj<GuiNode> {
        self.0.clone()
    }
}
