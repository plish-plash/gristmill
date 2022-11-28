use crate::{
    widget::{Widget, WidgetInput},
    Gui, GuiDraw, GuiLayout, GuiNode, GuiNodeExt, GuiNodeObj, WidgetBehavior,
};
use std::sync::Arc;

struct PanelBehavior(GuiNodeObj);

impl WidgetBehavior for PanelBehavior {
    fn update(&self, _input: WidgetInput) {
        // Changes to flags don't propagate until next frame.
        self.0.write().flags.visible = false;
    }
}

pub struct Panel(GuiNodeObj, Arc<PanelBehavior>);

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
        "Panel"
    }
    fn new(gui: &mut Gui, parent: GuiNodeObj) -> Self {
        let node = parent.add_child(GuiNode {
            layout: GuiLayout::fill(),
            ..Default::default()
        });
        let behavior = Arc::new(PanelBehavior(node.clone()));
        gui.register_behavior(behavior.clone());
        Panel(node, behavior)
    }
    fn node(&self) -> &GuiNodeObj {
        &self.0
    }
}
