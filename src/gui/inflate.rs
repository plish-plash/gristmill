use std::sync::Arc;
use serde::Deserialize;
use crate::color::Color;
use super::{Style, Gui, Node, text::Text, color_rect::ColorRect, button::{Button, ButtonStyle}, signal::{Signal, SignalTarget, SignalIdentifier}};

// TODO
#[derive(Clone)]
pub struct InflaterStyle {
    pub button_style: ButtonStyle,
}

#[derive(Deserialize)]
enum InflaterWidget {
    Text((f32, f32, f32, f32), Option<String>, f32, String),
    ColorRect((f32, f32, f32, f32)),
    Button(String, Option<String>),
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct InflaterNode {
    widget: Option<InflaterWidget>,
    style: Style,
    children: Vec<InflaterNode>,
}

impl InflaterNode {
    fn inflate(self, gui: &mut Gui, parent: Node, style: &InflaterStyle, signal_target: Option<Arc<SignalTarget>>) {
        let node = match self.widget {
            None => gui.add_child(parent),
            Some(InflaterWidget::Text(color, font, size, text)) => {
                let mut widget = Text::new();
                widget.set_text_all(font, size, text);
                widget.set_color(Color::from_components(color));
                gui.add_child_widget(parent, widget)
            }
            Some(InflaterWidget::ColorRect(color)) => {
                gui.add_child_widget(parent, ColorRect::new(Color::from_components(color)))
            }
            Some(InflaterWidget::Button(text, signal)) => {
                let signal = signal_target.as_ref().and_then(|t| signal.map(|s| Signal::new(t.clone(), SignalIdentifier::OwnedString(s))));
                gui.add_child_widget(parent, Button::new(&style.button_style, text, signal))
            }
        };
        gui.set_style(node, self.style);
        for child in self.children {
            child.inflate(gui, node, style, signal_target.clone());
        }
    }
}

pub fn inflate_gui<P>(gui: &mut Gui, parent: Node, style: &InflaterStyle, signal_target: Option<Arc<SignalTarget>>, path: P) where P: AsRef<std::path::Path> {
    // let nodes: Vec<InflaterNode> = crate::read_ron_file(path).unwrap(); // TODO handle errors
    // for node in nodes {
    //     node.inflate(gui, parent, style, signal_target.clone());
    // }
}

pub struct RootInflaterSignalHandler {
    style: InflaterStyle,
    signal_target: Option<Arc<SignalTarget>>,
}

impl RootInflaterSignalHandler {
    pub fn new(style: InflaterStyle, signal_target: Option<Arc<SignalTarget>>) -> RootInflaterSignalHandler {
        RootInflaterSignalHandler { style, signal_target }
    }
    pub fn process(&self, signal: String, gui: &mut Gui) {
        gui.remove_all();
        inflate_gui(gui, gui.root_node(), &self.style, self.signal_target.clone(), signal);
    }
}
