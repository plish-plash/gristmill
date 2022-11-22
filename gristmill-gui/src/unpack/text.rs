use gristmill::Obj;
use serde::{Deserialize, Serialize};

use super::{PackedWidget, Unpacker};
use crate::{
    widget::{Text, TextAlign, Widget},
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
        let mut text = Text::create(gui, parent, self.class.as_deref());
        text.set_text_string(&self.text);
        if let Some(align) = self.align {
            text.set_align(align);
        }
        unpacker.finish_widget(text, &self.name, &self.layout)
    }
}
