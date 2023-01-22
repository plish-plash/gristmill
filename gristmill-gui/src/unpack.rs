use crate::{
    widget::{StyleValue, Widget, WidgetNode},
    Gui, GuiLayout, GuiNode, GuiNodeExt, GuiNodeId,
};
use gristmill_core::asset::{Asset, AssetCategory, AssetResult, AssetStorage, BufReader};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, rc::Rc};

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct PackedNode {
    r#type: String,
    name: String,
    class: Option<String>,
    layout: Option<GuiLayout>,
    children: Vec<PackedNode>,
    #[serde(flatten)]
    extra: HashMap<String, StyleValue>,
}

impl Asset for PackedNode {
    fn category() -> AssetCategory {
        AssetCategory::ASSET
    }
    fn read_from(reader: BufReader) -> AssetResult<Self> {
        gristmill_core::asset::util::read_yaml(reader)
    }
}

impl PackedNode {
    fn unpack(
        &self,
        gui: &mut Gui,
        parent: GuiNodeId,
        widgets: &mut UnpackedWidgets,
    ) -> Option<GuiNodeId> {
        if let Some(type_unpacker) = gui.unpacker.types.get(&self.r#type) {
            let widget = type_unpacker(gui, parent, self.class.as_deref());
            if let Some(layout) = self.layout {
                widget.set_layout(gui, layout);
            }
            widget.unpack_extra_fields(gui, &self.extra);
            let widget_node = widget.node();
            if !self.name.is_empty() {
                widgets.1.insert(self.name.clone(), widget);
            }
            for child in self.children.iter() {
                child.unpack(gui, widget_node, widgets);
            }
            Some(widget_node)
        } else {
            log::error!("Unable to unpack widget of type {}.", &self.r#type);
            None
        }
    }
}

type CreateWidgetFn = fn(&mut Gui, GuiNodeId, Option<&str>) -> Box<dyn WidgetNode>;

#[derive(Default)]
pub(crate) struct Unpacker {
    types: HashMap<String, CreateWidgetFn>,
    storage: AssetStorage<Rc<PackedNode>>,
}

impl Unpacker {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn with_standard_widgets() -> Self {
        use crate::widget::*;
        let mut unpacker = Self::new();
        unpacker.types.insert(String::new(), |gui, parent, _class| {
            Box::new(parent.add_child(gui, GuiNode::default()))
        });
        unpacker.register_widget::<Button>();
        unpacker.register_widget::<Image>();
        unpacker.register_widget::<Panel>();
        unpacker.register_widget::<Text>();
        unpacker
    }

    pub fn register_widget<T: Widget + WidgetNode>(&mut self) {
        self.types
            .insert(T::type_name().to_owned(), |gui, parent, class| {
                Box::new(gui.create_widget::<T>(parent, class))
            });
    }
}

#[derive(Default)]
pub struct UnpackedWidgets(Option<GuiNodeId>, HashMap<String, Box<dyn WidgetNode>>);

impl UnpackedWidgets {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn root(&self) -> Option<GuiNodeId> {
        self.0
    }
    pub fn get<W: WidgetNode>(&mut self, name: &str) -> Option<W> {
        if let Some(widget) = self.1.remove(name) {
            if let Ok(cast) = widget.as_any_box().downcast::<W>() {
                Some(*cast)
            } else {
                log::error!("Widget {} is wrong type.", name);
                None
            }
        } else {
            log::error!("No widget named {}.", name);
            None
        }
    }
}

pub trait PackedWidget: Sized {
    fn asset_path() -> &'static str;
    fn new(widgets: UnpackedWidgets) -> Option<Self>;
    fn load(gui: &mut Gui, parent: GuiNodeId) -> Option<Self> {
        let mut widgets = UnpackedWidgets::new();
        let packed_node = gui.unpacker.storage.load(Self::asset_path())?;
        widgets.0 = packed_node.unpack(gui, parent, &mut widgets);
        Self::new(widgets)
    }
}
