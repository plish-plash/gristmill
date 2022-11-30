use super::{PackedWidget, Unpacker};
use crate::{widget::Button, Gui, GuiLayout, GuiNodeObj};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct PackedButton {
    name: Option<String>,
    class: Option<String>,
    layout: Option<GuiLayout>,
    label: String,
}

impl PackedWidget for PackedButton {
    fn unpack(&self, unpacker: &mut Unpacker, gui: &mut Gui, parent: GuiNodeObj) -> GuiNodeObj {
        let button: Button = gui.create_widget(parent, self.class.as_deref());
        button.set_label_string(&self.label);
        unpacker.finish_widget(button, &self.name, &self.layout)
    }
}
