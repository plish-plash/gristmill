use crate::{
    widget::{StyleQuery, Widget, WidgetBehavior, WidgetInput, WidgetNode},
    Gui, GuiDraw, GuiLayout, GuiNode, GuiNodeExt, GuiNodeId, GuiNodeStorage,
};
use std::{any::Any, rc::Rc};

struct PanelBehavior(GuiNodeId);

impl WidgetBehavior for PanelBehavior {
    fn update(&self, nodes: &mut GuiNodeStorage, _input: &WidgetInput) {
        // Changes to flags don't propagate until next frame.
        if let Some(node) = nodes.get_mut(self.0) {
            node.flags.visible = false;
        }
    }
}

pub struct Panel(GuiNodeId, Rc<PanelBehavior>);

impl Panel {
    pub fn show(&self, gui: &mut Gui) {
        self.set_visible(gui, true);
    }
    pub fn set_background(&self, gui: &mut Gui, draw: GuiDraw) {
        if let Some(node) = self.node_data(gui) {
            node.draw = draw;
        }
    }
}

impl Widget for Panel {
    fn type_name() -> &'static str {
        "Panel"
    }
    fn new(gui: &mut Gui, parent: GuiNodeId, _style: StyleQuery) -> Self {
        let node = parent.add_child(
            gui,
            GuiNode {
                layout: GuiLayout::fill(),
                ..Default::default()
            },
        );
        let behavior = gui.register_behavior(PanelBehavior(node));
        Panel(node, behavior)
    }
}

impl WidgetNode for Panel {
    fn as_any_box(self: Box<Self>) -> Box<dyn Any> {
        self
    }
    fn node(&self) -> GuiNodeId {
        self.0
    }
}
