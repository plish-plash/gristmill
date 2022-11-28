use super::{PackedWidget, Unpacker};
use crate::{
    widget::{Image, Widget},
    Gui, GuiLayout, GuiNodeObj,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PackedImage<W: PackedWidget> {
    name: Option<String>,
    class: Option<String>,
    layout: Option<GuiLayout>,
    #[serde(default = "Vec::new")]
    children: Vec<W>,
}

impl<W: PackedWidget> PackedWidget for PackedImage<W> {
    fn unpack(&self, unpacker: &mut Unpacker, gui: &mut Gui, parent: GuiNodeObj) -> GuiNodeObj {
        let image: Image = gui.create_widget(parent);
        unpacker.unpack_children(gui, image.node(), &self.children);
        unpacker.finish_widget(image, &self.name, &self.layout)
    }
}
