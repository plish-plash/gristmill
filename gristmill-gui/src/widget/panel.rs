use crate::{
    widget::{Widget, WidgetInput},
    Gui, GuiDraw, GuiLayout, GuiNode, GuiNodeExt, GuiNodeObj, WidgetState,
};
use std::sync::{Arc, RwLock};

struct PanelState(GuiNodeObj);

impl WidgetState for PanelState {
    fn update(&mut self, _input: WidgetInput) {
        // Changes to flags don't propagate until next frame.
        self.0.write().flags.visible = false;
    }
}

pub struct Panel(GuiNodeObj, Arc<RwLock<PanelState>>);

impl Panel {
    pub fn show(&self) {
        self.set_visible(true);
    }
    pub fn set_background(&self, draw: GuiDraw) {
        self.0.write().draw = draw;
    }
}

impl Widget for Panel {
    fn class_name() -> &'static str {
        "panel"
    }
    fn new(gui: &mut Gui, parent: GuiNodeObj) -> Self {
        let node = parent.add_child(GuiNode {
            layout: GuiLayout::fill(),
            ..Default::default()
        });
        let state = gui.register_widget_state(PanelState(node.clone()));
        Panel(node, state)
    }
    fn node(&self) -> &GuiNodeObj {
        &self.0
    }
}
