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
use gristmill::asset::{Asset, AssetResult, BufReader};
use serde::{Deserialize, Serialize};
use std::{any::Any, collections::HashMap};

#[derive(Default)]
pub struct Unpacker(HashMap<String, Box<dyn Any>>);

impl Unpacker {
    pub fn get_widget<W>(&mut self, name: &str) -> Option<Box<W>>
    where
        W: Widget + 'static,
    {
        if let Some(widget) = self.0.remove(name) {
            if let Ok(cast) = widget.downcast() {
                Some(cast)
            } else {
                log::error!("Widget {} is wrong type.", name);
                None
            }
        } else {
            log::error!("No widget named {}.", name);
            None
        }
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
            self.0.insert(name.to_owned(), Box::new(widget));
        }
        node
    }
}

pub trait PackedWidget: Clone {
    fn unpack(&self, unpacker: &mut Unpacker, gui: &mut Gui, parent: GuiNodeObj) -> GuiNodeObj;
}

#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Clone, Serialize, Deserialize)]
pub struct PackedPanel<W: PackedWidget> {
    name: Option<String>,
    class: Option<String>,
    layout: Option<GuiLayout>,
    children: Vec<W>,
}

impl<W: PackedWidget> PackedWidget for PackedPanel<W> {
    fn unpack(&self, unpacker: &mut Unpacker, gui: &mut Gui, parent: GuiNodeObj) -> GuiNodeObj {
        let panel: Panel = gui.create_widget(parent, self.class.as_deref());
        unpacker.unpack_children(gui, panel.node(), &self.children);
        unpacker.finish_widget(panel, &self.name, &self.layout)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum StandardPackedWidget {
    Node(PackedNode<StandardPackedWidget>),
    Panel(PackedPanel<StandardPackedWidget>),
    Image(PackedImage<StandardPackedWidget>),
    Text(PackedText),
    Button(PackedButton),
}

impl PackedWidget for StandardPackedWidget {
    fn unpack(&self, unpacker: &mut Unpacker, gui: &mut Gui, parent: GuiNodeObj) -> GuiNodeObj {
        match self {
            StandardPackedWidget::Node(inner) => inner.unpack(unpacker, gui, parent),
            StandardPackedWidget::Panel(inner) => inner.unpack(unpacker, gui, parent),
            StandardPackedWidget::Image(inner) => inner.unpack(unpacker, gui, parent),
            StandardPackedWidget::Text(inner) => inner.unpack(unpacker, gui, parent),
            StandardPackedWidget::Button(inner) => inner.unpack(unpacker, gui, parent),
        }
    }
}

impl Asset for StandardPackedWidget {
    fn read_from(reader: BufReader) -> AssetResult<Self> {
        gristmill::asset::util::read_yaml(reader)
    }
}
