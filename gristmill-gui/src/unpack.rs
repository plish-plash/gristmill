use crate::{
    widget::{StyleValues, Widget, WidgetNode},
    Gui, GuiNode, GuiNodeExt, GuiNodeId,
};
use gristmill_core::asset::{self, AssetError, AssetResult};
use serde::Deserialize;
use std::{collections::HashMap, rc::Rc};

#[derive(Default, Deserialize)]
#[serde(default)]
struct PackedNode {
    r#type: String,
    name: String,
    class: Vec<String>,
    children: Vec<PackedNode>,
    #[serde(flatten)]
    extra: StyleValues,
}

impl PackedNode {
    fn load_asset(file: &str) -> AssetResult<Self> {
        let mut packed: PackedNode = asset::load_yaml_file("assets", file)?;
        if !packed.r#type.is_empty() {
            packed.class.insert(0, packed.r#type.clone());
        }
        Ok(packed)
    }

    fn unpack_widget<W: Widget>(&self, gui: &mut Gui, parent: GuiNodeId) -> W {
        let mut style = gui.styles.query(self.class.iter().map(|s| -> &str { s }));
        style.extend(self.extra.clone());
        W::new(gui, parent, style)
    }
    fn unpack(
        &self,
        gui: &mut Gui,
        parent: GuiNodeId,
        widgets: &mut UnpackedWidgets,
    ) -> AssetResult<GuiNodeId> {
        if let Some(type_unpacker) = gui.unpacker.types.get(&self.r#type) {
            let widget = type_unpacker(gui, parent, self);
            let widget_node = widget.node();
            if !self.name.is_empty() {
                widgets.1.insert(self.name.clone(), widget);
            }
            for child in self.children.iter() {
                child.unpack(gui, widget_node, widgets)?;
            }
            Ok(widget_node)
        } else {
            Err(AssetError::Other(format!(
                "unknown widget type {}",
                &self.r#type
            )))
        }
    }
}

type CreateWidgetFn = fn(&mut Gui, GuiNodeId, &PackedNode) -> Box<dyn WidgetNode>;

#[derive(Default)]
pub(crate) struct Unpacker {
    types: HashMap<String, CreateWidgetFn>,
    packed_node_cache: HashMap<String, Rc<PackedNode>>,
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
            .insert(T::class_name().to_owned(), |gui, parent, packed| {
                Box::new(packed.unpack_widget::<T>(gui, parent))
            });
    }

    fn load_packed_node(&mut self, file: &str) -> AssetResult<Rc<PackedNode>> {
        if let Some(packed) = self.packed_node_cache.get(file) {
            Ok(packed.clone())
        } else {
            let packed = Rc::new(PackedNode::load_asset(file)?);
            self.packed_node_cache
                .insert(file.to_owned(), packed.clone());
            Ok(packed)
        }
    }
}

#[derive(Default)]
pub struct UnpackedWidgets(GuiNodeId, HashMap<String, Box<dyn WidgetNode>>);

impl UnpackedWidgets {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn root(&self) -> GuiNodeId {
        self.0
    }
    pub fn get<W: WidgetNode>(&mut self, name: &str) -> AssetResult<W> {
        if let Some(widget) = self.1.remove(name) {
            if let Ok(cast) = widget.as_any_box().downcast::<W>() {
                Ok(*cast)
            } else {
                Err(AssetError::Other(format!("widget {name} is wrong type")))
            }
        } else {
            Err(AssetError::Other(format!("no widget named {name}")))
        }
    }
}

pub trait PackedWidget: Sized {
    fn asset_path() -> &'static str;
    fn new(widgets: UnpackedWidgets) -> AssetResult<Self>;
    fn load_asset(gui: &mut Gui, parent: GuiNodeId) -> AssetResult<Self> {
        let mut widgets = UnpackedWidgets::new();
        let packed = gui.unpacker.load_packed_node(Self::asset_path())?;
        widgets.0 = packed.unpack(gui, parent, &mut widgets)?;
        Self::new(widgets)
    }
}
