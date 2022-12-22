use crate::{
    widget::{StyleQuery, StyleValue, StyleValues, Widget, WidgetNode},
    Gui, GuiDraw, GuiLayout, GuiNode, GuiNodeExt, GuiNodeId,
};
use glyph_brush::*;
use gristmill_core::{
    geom2d::{IRect, Size},
    math::IVec2,
};
use serde::{Deserialize, Serialize};
use std::any::Any;

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
    pub color: gristmill_core::Color,
}

impl Default for TextStyle {
    fn default() -> TextStyle {
        TextStyle {
            font: FontId::default(),
            font_size: 18,
            color: gristmill_core::Color::BLACK,
        }
    }
}

pub struct Text {
    style: TextStyle,
    node: GuiNodeId,
}

impl Text {
    pub fn set_text(&self, gui: &mut Gui, text: Vec<OwnedText>) {
        if let Some(node) = self.node_data(gui) {
            if let GuiDraw::Text(section) = &mut node.draw {
                section.text = text;
            }
        }
    }
    pub fn set_text_string<S>(&self, gui: &mut Gui, text: S)
    where
        S: Into<String>,
    {
        let text = OwnedText::new(text)
            .with_font_id(self.style.font)
            .with_scale(self.style.font_size as f32)
            .with_color(<[f32; 4]>::from(self.style.color));
        self.set_text(gui, vec![text]);
    }
    pub fn set_align(&self, gui: &mut Gui, align: TextAlign) {
        if let Some(node) = self.node_data(gui) {
            if let GuiDraw::Text(section) = &mut node.draw {
                section.layout = align.into();
            }
        }
    }

    pub(crate) fn default_style() -> StyleValues {
        let default = TextStyle::default();
        let mut style = StyleValues::new();
        style.insert("font_size".to_owned(), StyleValue::from(default.font_size));
        style.insert(
            "color".to_owned(),
            StyleValue::try_from(<[f32; 4]>::from(default.color)).unwrap(),
        );
        style
    }
}

impl Widget for Text {
    fn type_name() -> &'static str {
        "Text"
    }
    fn new(gui: &mut Gui, parent: GuiNodeId, style: StyleQuery) -> Self {
        let mut text_style = TextStyle::default(); // TODO font
        text_style.font_size = style.get("font_size").unwrap_or(text_style.font_size);
        text_style.color = style.get("color").unwrap_or(text_style.color);
        let node = parent.add_child(
            gui,
            GuiNode::with_draw_and_layout(
                GuiDraw::Text(OwnedSection::default()),
                GuiLayout::Child(IRect::new(IVec2::ZERO, Size::new(256, 32))),
            ),
        );
        Text {
            style: text_style,
            node,
        }
    }
}

impl WidgetNode for Text {
    fn as_any_box(self: Box<Self>) -> Box<dyn Any> {
        self
    }
    fn node(&self) -> GuiNodeId {
        self.node
    }
}
