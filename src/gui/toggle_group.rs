use stretch::style::FlexDirection;
use super::*;

pub struct ToggleGroup {
    modification_queue: Arc<GuiModifierQueue>,
    button_signal_target: Arc<signal::SignalTarget>,
    selected_signal: Option<Signal>,
    buttons: Vec<Node>,
    selected_index: usize,
}

impl ToggleGroup {
    pub fn button_row(button_style: button::ButtonStyle, buttons: Vec<String>, selected_signal: Option<Signal>) -> ButtonRowToggleGroupBuilder {
        ButtonRowToggleGroupBuilder(button_style, buttons, selected_signal)
    }

    fn update_selected(&self) {
        self.modification_queue.enqueue(button::ButtonToggledModifier::new(self.buttons[self.selected_index], true));
        if let Some(signal) = self.selected_signal.as_ref() {
            signal.send_value(self.selected_index);
        }
    }
    fn set_selected_index(&mut self, index: usize) {
        if self.selected_index == index { return; }
        self.modification_queue.enqueue(button::ButtonToggledModifier::new(self.buttons[self.selected_index], false));
        self.selected_index = index;
        self.update_selected();
    }
}

impl Widget for ToggleGroup {
    fn as_any(&mut self) -> &mut dyn Any { self }
    fn refresh_drawables(&mut self, _context: &mut DrawContext) {
        let mut new_selected_index = self.selected_index;
        self.button_signal_target.process(|identifier, _| {
            new_selected_index = identifier.index();
        });
        self.set_selected_index(new_selected_index);
    }
    fn draw(&self, _rect: Rect) -> Option<DrawCommand> {
        None
    }
}

pub struct ButtonRowToggleGroupBuilder(button::ButtonStyle, Vec<String>, Option<Signal>);

impl WidgetBuilder for ButtonRowToggleGroupBuilder {
    type Widget = ToggleGroup;
    fn build_widget(self, gui: &mut Gui, node: Node) -> ToggleGroup {
        gui.set_style(node, Style {
            flex_direction: FlexDirection::Row,
            ..Default::default()
        });
        let button_signal_target = signal::SignalTarget::new();
        let mut buttons = Vec::new();
        for (index, button_text) in self.1.into_iter().enumerate() {
            buttons.push(gui.add_child_widget(node, button::Button::new(&self.0, button_text, Some(Signal::new_index(button_signal_target.clone(), index)))));
        }
        let group = ToggleGroup {
            modification_queue: gui.modification_queue(),
            button_signal_target,
            selected_signal: self.2,
            buttons,
            selected_index: 0,
        };
        group.update_selected();
        group
    }
}
