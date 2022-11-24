use crate::{
    widget::{StyleQuery, StyleValues, Widget},
    Gui, GuiDraw, GuiLayout, GuiNode, GuiNodeExt,
};
use glyph_brush::*;
use gristmill::{
    color::Pixel,
    geom2d::{Rect, Size},
    math::IVec2,
    Obj,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Middle,
    MiddleLeft,
    MiddleRight,
    LeftWrap,
    RightWrap,
    CenterWrap,
}

impl From<TextAlign> for Layout<BuiltInLineBreaker> {
    fn from(align: TextAlign) -> Self {
        match align {
            TextAlign::Left => Layout::default_single_line().h_align(HorizontalAlign::Left),
            TextAlign::Right => Layout::default_single_line().h_align(HorizontalAlign::Right),
            TextAlign::Center => Layout::default_single_line().h_align(HorizontalAlign::Center),
            TextAlign::Middle => Layout::SingleLine {
                h_align: HorizontalAlign::Center,
                v_align: VerticalAlign::Center,
                line_breaker: Default::default(),
            },
            TextAlign::MiddleLeft => Layout::SingleLine {
                h_align: HorizontalAlign::Left,
                v_align: VerticalAlign::Center,
                line_breaker: Default::default(),
            },
            TextAlign::MiddleRight => Layout::SingleLine {
                h_align: HorizontalAlign::Right,
                v_align: VerticalAlign::Center,
                line_breaker: Default::default(),
            },
            TextAlign::LeftWrap => Layout::default_wrap().h_align(HorizontalAlign::Left),
            TextAlign::RightWrap => Layout::default_wrap().h_align(HorizontalAlign::Right),
            TextAlign::CenterWrap => Layout::default_wrap().h_align(HorizontalAlign::Center),
        }
    }
}

struct TextStyle {
    pub font: FontId,
    pub font_size: i32,
    pub color: gristmill::Color,
}

impl Default for TextStyle {
    fn default() -> TextStyle {
        TextStyle {
            font: FontId::default(),
            font_size: 18,
            color: crate::color::BLACK,
        }
    }
}

pub struct Text {
    style: TextStyle,
    node: Obj<GuiNode>,
}

impl Text {
    pub fn set_text(&self, text: Vec<OwnedText>) {
        if let GuiDraw::Text(section) = &mut self.node.write().draw {
            section.text = text;
        }
    }
    pub fn set_text_string<S>(&self, text: S)
    where
        S: Into<String>,
    {
        let text = OwnedText::new(text)
            .with_font_id(self.style.font)
            .with_scale(self.style.font_size as f32)
            .with_color(self.style.color.into_raw::<[f32; 4]>());
        self.set_text(vec![text]);
    }
    pub fn set_align(&self, align: TextAlign) {
        if let GuiDraw::Text(section) = &mut self.node.write().draw {
            section.layout = align.into();
        }
    }

    pub(crate) fn default_style() -> StyleValues {
        let default = TextStyle::default();
        let mut style = StyleValues::new();
        style.set("font-size", default.font_size);
        style.set("color", default.color);
        style
    }
}

impl Widget for Text {
    fn class_name() -> &'static str {
        "Text"
    }
    fn new(_gui: &mut Gui, parent: Obj<GuiNode>) -> Self {
        let node = parent.add_child(GuiNode::with_draw_and_layout(
            GuiDraw::Text(OwnedSection::default()),
            GuiLayout::Child(Rect::new(IVec2::ZERO, Size::new(256, 32))),
        ));
        Text {
            style: TextStyle::default(),
            node,
        }
    }
    fn apply_style(&mut self, style: StyleQuery) {
        let default = TextStyle::default();
        // TODO font
        self.style.font_size = style.get("font-size", default.font_size);
        self.style.color = style.get("color", default.color);
    }
    fn node(&self) -> Obj<GuiNode> {
        self.node.clone()
    }
}
