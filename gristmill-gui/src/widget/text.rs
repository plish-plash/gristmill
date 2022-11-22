use crate::widget::{StyleValue, StyleValues, Widget, WidgetType};
use crate::{Gui, GuiDraw, GuiLayout, GuiNode, GuiNodeExt};
use glyph_brush::*;
use gristmill::color::Pixel;
use gristmill::geom2d::{Rect, Size};
use gristmill::math::IVec2;
use gristmill::Obj;
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

pub struct TextStyle {
    pub font: FontId,
    pub font_size: i32,
    pub align: TextAlign,
    pub color: gristmill::Color,
}

impl TextStyle {
    fn from_style(style: &StyleValues) -> TextStyle {
        TextStyle {
            font_size: style
                .get("font-size")
                .and_then(StyleValue::to_i32)
                .unwrap_or(18),
            color: style
                .get("color")
                .and_then(StyleValue::to_color)
                .unwrap_or(crate::color::BLACK),
            // TODO other fields
            ..Default::default()
        }
    }
}
impl Default for TextStyle {
    fn default() -> TextStyle {
        TextStyle {
            font: FontId::default(),
            font_size: 18,
            align: TextAlign::Left,
            color: crate::color::BLACK,
        }
    }
}

pub struct Text {
    style: TextStyle,
    node: Obj<GuiNode>,
}

impl Text {
    pub fn create_with_text_style(parent: Obj<GuiNode>, style: TextStyle) -> Text {
        let draw = GuiDraw::Text(OwnedSection::default().with_layout(style.align));
        let node = parent.add_child(GuiNode::with_draw_and_layout(
            draw,
            GuiLayout::Child(Rect::new(IVec2::ZERO, Size::new(256, 32))),
        ));
        Text { style, node }
    }

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
    pub fn set_align(&mut self, align: TextAlign) {
        self.style.align = align;
        if let GuiDraw::Text(section) = &mut self.node.write().draw {
            section.layout = align.into();
        }
    }
}

impl Widget for Text {
    fn widget_type() -> WidgetType {
        WidgetType::text()
    }
    fn create_with_style(_gui: &mut Gui, parent: Obj<GuiNode>, style: &StyleValues) -> Text {
        Self::create_with_text_style(parent, TextStyle::from_style(style))
    }
    fn node(&self) -> Obj<GuiNode> {
        self.node.clone()
    }
}
