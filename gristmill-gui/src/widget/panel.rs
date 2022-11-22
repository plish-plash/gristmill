use crate::widget::{InputState, StyleValues, Widget, WidgetType};
use crate::{Gui, GuiDraw, GuiLayout, GuiNode, GuiNodeExt, WidgetBehavior, WidgetObj};
use gristmill::Obj;

struct PanelBehavior(Obj<GuiNode>);

impl WidgetBehavior for PanelBehavior {
    fn node(&self) -> Obj<GuiNode> {
        self.0.clone()
    }
    fn update(&mut self, _state: InputState) {
        // Changes to flags don't propagate until next frame.
        self.0.write().flags.visible = false;
    }
}

pub struct Panel(Obj<GuiNode>, WidgetObj<PanelBehavior>);

impl Panel {
    pub fn show(&self) {
        self.set_visible(true);
    }
    pub fn set_background(&self, draw: GuiDraw) {
        self.0.write().draw = draw;
    }
}

impl Widget for Panel {
    fn widget_type() -> WidgetType {
        WidgetType::panel()
    }
    fn create_with_style(gui: &mut Gui, parent: Obj<GuiNode>, _style: &StyleValues) -> Panel {
        let node = parent.add_child(GuiNode {
            layout: GuiLayout::fill(),
            ..Default::default()
        });
        let behavior = gui.register_behavior(PanelBehavior(node.clone()));
        Panel(node, behavior)
    }
    fn node(&self) -> Obj<GuiNode> {
        self.0.clone()
    }
}
