use gristmill::Obj;
use serde::{Deserialize, Serialize};

use super::{PackedWidget, Unpacker};
use crate::{
    widget::{Text, TextAlign},
    Gui, GuiLayout, GuiNode,
};

#[derive(Serialize, Deserialize)]
pub struct PackedText {
    name: Option<String>,
    class: Option<String>,
    layout: Option<GuiLayout>,
    text: String,
    align: Option<TextAlign>,
}

impl PackedWidget for PackedText {
    fn unpack(&self, unpacker: &mut Unpacker, gui: &mut Gui, parent: Obj<GuiNode>) -> Obj<GuiNode> {
        let text: Text = gui.create_widget(parent);
        text.set_text_string(&self.text);
        if let Some(align) = self.align {
            text.set_align(align);
        }
        unpacker.finish_widget(text, &self.name, &self.layout)
    }
}
