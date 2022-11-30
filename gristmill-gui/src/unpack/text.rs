use super::{PackedWidget, Unpacker};
use crate::{
    widget::{Text, TextAlign},
    Gui, GuiLayout, GuiNodeObj,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct PackedText {
    name: Option<String>,
    class: Option<String>,
    layout: Option<GuiLayout>,
    text: String,
    align: Option<TextAlign>,
}

impl PackedWidget for PackedText {
    fn unpack(&self, unpacker: &mut Unpacker, gui: &mut Gui, parent: GuiNodeObj) -> GuiNodeObj {
        let text: Text = gui.create_widget(parent, self.class.as_deref());
        text.set_text_string(&self.text);
        if let Some(align) = self.align {
            text.set_align(align);
        }
        unpacker.finish_widget(text, &self.name, &self.layout)
    }
}
