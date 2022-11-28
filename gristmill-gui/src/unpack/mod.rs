mod button;
mod image;
mod text;

pub use button::*;
pub use image::*;
pub use text::*;

use crate::{
    widget::{Panel, Widget},
    Gui, GuiLayout, GuiNode, GuiNodeExt, GuiNodeObj,
};
use serde::{Deserialize, Serialize};
use std::{any::Any, collections::HashMap};

pub struct Unpacker(HashMap<String, Option<Box<dyn Any>>>);

impl Unpacker {
    fn add<S, W>(&mut self, name: S, widget: W)
    where
        S: Into<String>,
        W: Widget + 'static,
    {
        self.0.insert(name.into(), Some(Box::new(widget)));
    }
    pub fn named_widget<W>(&mut self, name: &str) -> W
    where
        W: Widget + 'static,
    {
        *self
            .0
            .get_mut(name)
            .expect("missing widget")
            .take()
            .expect("widget already taken")
            .downcast()
            .expect("widget wrong type")
    }
    pub fn named_widget_array<W>(&mut self, name: &str) -> Vec<W>
    where
        W: Widget + 'static,
    {
        let mut items = Vec::new();
        loop {
            let key = format!("{}[{}]", name, items.len());
            if self.0.contains_key(&key) {
                items.push(self.named_widget(&key));
            } else {
                break;
            }
        }
        items
    }

    pub fn unpack_children<P>(&mut self, gui: &mut Gui, node: &GuiNodeObj, children: &[P])
    where
        P: PackedWidget,
    {
        for child in children.iter() {
            child.unpack(self, gui, node.clone());
        }
    }
    pub fn finish_widget<W>(
        &mut self,
        widget: W,
        name: &Option<String>,
        layout: &Option<GuiLayout>,
    ) -> GuiNodeObj
    where
        W: Widget + 'static,
    {
        if let Some(l) = *layout {
            widget.set_layout(l);
        }
        let node = widget.node().clone();
        if let Some(name) = name.as_deref() {
            self.add(name, widget);
        }
        node
    }
}

pub trait PackedWidget {
    fn unpack(&self, unpacker: &mut Unpacker, gui: &mut Gui, parent: GuiNodeObj) -> GuiNodeObj;
}

impl PackedWidget for () {
    fn unpack(&self, _unpacker: &mut Unpacker, _gui: &mut Gui, _parent: GuiNodeObj) -> GuiNodeObj {
        panic!("unpacking nonexistent node");
    }
}

#[derive(Serialize, Deserialize)]
pub struct PackedNode<W: PackedWidget> {
    layout: GuiLayout,
    children: Vec<W>,
}

impl<W: PackedWidget> PackedWidget for PackedNode<W> {
    fn unpack(&self, unpacker: &mut Unpacker, gui: &mut Gui, parent: GuiNodeObj) -> GuiNodeObj {
        let node = parent.add_child(GuiNode::with_layout(self.layout));
        unpacker.unpack_children(gui, &node, &self.children);
        node
    }
}

#[derive(Serialize, Deserialize)]
pub struct PackedPanel<W: PackedWidget> {
    name: Option<String>,
    class: Option<String>,
    layout: Option<GuiLayout>,
    children: Vec<W>,
}

impl<W: PackedWidget> PackedWidget for PackedPanel<W> {
    fn unpack(&self, unpacker: &mut Unpacker, gui: &mut Gui, parent: GuiNodeObj) -> GuiNodeObj {
        let panel: Panel = gui.create_widget(parent);
        unpacker.unpack_children(gui, panel.node(), &self.children);
        unpacker.finish_widget(panel, &self.name, &self.layout)
    }
}

pub trait WidgetCollection: Sized {
    fn from_unpacked_widgets(root: GuiNodeObj, unpacker: Unpacker) -> Self;
    fn unpack<W: PackedWidget>(gui: &mut Gui, parent: GuiNodeObj, packed_widget: W) -> Self {
        let mut unpacker = Unpacker(HashMap::new());
        let root = packed_widget.unpack(&mut unpacker, gui, parent);
        Self::from_unpacked_widgets(root, unpacker)
    }
}
