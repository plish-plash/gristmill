use crate::{
    widget::{StyleValues, Widget, WidgetNode, WidgetNodeExt, WidgetStyle},
    Anchor, Gui, GuiNode, GuiNodeExt, GuiNodeId, NodeDraw,
};
use glyph_brush::*;
use std::any::Any;

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
    fn make_layout(h_align: Anchor, v_align: Anchor, wrap: bool) -> Layout<BuiltInLineBreaker> {
        let h_align = match h_align {
            Anchor::Begin => HorizontalAlign::Left,
            Anchor::Middle => HorizontalAlign::Center,
            Anchor::End => HorizontalAlign::Right,
        };
        let v_align = match v_align {
            Anchor::Begin => VerticalAlign::Top,
            Anchor::Middle => VerticalAlign::Center,
            Anchor::End => VerticalAlign::Bottom,
        };
        if wrap {
            Layout::Wrap {
                line_breaker: Default::default(),
                h_align,
                v_align,
            }
        } else {
            Layout::SingleLine {
                line_breaker: Default::default(),
                h_align,
                v_align,
            }
        }
    }

    pub fn set_text(&self, gui: &mut Gui, text: Vec<OwnedText>) {
        if let Some(node) = self.node_data(gui) {
            if let NodeDraw::Text(section) = &mut node.draw {
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
    pub fn set_text_align(&self, gui: &mut Gui, align: (Anchor, Anchor), wrap: bool) {
        if let Some(node) = self.node_data(gui) {
            if let NodeDraw::Text(section) = &mut node.draw {
                section.layout = Self::make_layout(align.0, align.1, wrap)
            }
        }
    }
}

impl Widget for Text {
    fn class_name() -> &'static str {
        "text"
    }
    fn new(gui: &mut Gui, parent: GuiNodeId, mut style: StyleValues) -> Self {
        let mut text_style = TextStyle::default(); // TODO font
        text_style.font_size = style.widget_value("font_size", text_style.font_size);
        text_style.color = style.widget_value("color", text_style.color);
        let h_align = style.widget_value("halign", Anchor::Begin);
        let v_align = style.widget_value("valign", Anchor::Begin);
        let wrap = style.widget_value("wrap", false);
        let text = style.widget_value("text", String::new());
        let node = parent.add_child(
            gui,
            GuiNode::new(
                style.widget_layout(),
                NodeDraw::Text(
                    OwnedSection::default().with_layout(Self::make_layout(h_align, v_align, wrap)),
                ),
            ),
        );
        let widget = Text {
            style: text_style,
            node,
        };
        widget.set_text_string(gui, text);
        widget
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
