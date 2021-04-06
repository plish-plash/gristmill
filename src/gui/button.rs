use palette::Shade;
use super::*;

#[derive(Clone, Debug)]
pub struct ButtonStyle {
    pub font: Option<String>,
    pub padding: f32,
    pub text_size: f32,
    pub text_color: Color,
    pub normal_color: Color,
    pub toggled_color: Color,
    pub disabled_color: Color,
    pub hover_shade: f32,
    pub press_shade: f32,
}

impl ButtonStyle {
    fn get_color(&self, state: ButtonState, toggled: bool) -> Color {
        let color = match state {
            ButtonState::Disabled => self.disabled_color,
            _ => if toggled { self.toggled_color } else { self.normal_color },
        };
        match state {
            ButtonState::Hovered => color.darken(self.hover_shade),
            ButtonState::Pressed => color.darken(self.press_shade),
            _ => color,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
    Disabled,
}

pub struct Button {
    modification_queue: Arc<GuiModifierQueue>,
    pressed_signal: Option<Signal>,
    style: ButtonStyle,
    pressed: bool,
    toggled: bool,
    image: Node,
    text: Node,
    state: ButtonState,
}

impl Button {
    pub fn new(style: &ButtonStyle, text: String, pressed_signal: Option<Signal>) -> ButtonBuilder {
        ButtonBuilder(style.clone(), text, pressed_signal)
    }

    fn set_state(&mut self, state: ButtonState) {
        if self.state == state { return; }
        self.state = state;
        self.modification_queue.enqueue(color_rect::ColorRectModifier::new(self.image, self.style.get_color(state, self.toggled)));
    }
    fn set_toggled(&mut self, toggled: bool) {
        if self.toggled == toggled { return; }
        self.toggled = toggled;
        self.modification_queue.enqueue(color_rect::ColorRectModifier::new(self.image, self.style.get_color(self.state, toggled)));
    }
}

impl Widget for Button {
    fn as_any(&mut self) -> &mut dyn Any { self }
    fn refresh_drawables(&mut self, _context: &mut DrawContext) {}
    fn draw(&self, _rect: Rect) -> Option<DrawCommand> {
        None
    }
    
    fn cursor_input(&mut self, input: &CursorAction, cursor_over: bool) {
        if cursor_over {
            if input.pressed() {
                self.pressed = true;
            }
            else if self.pressed && input.released() {
                if let Some(signal) = self.pressed_signal.as_ref() {
                    signal.send();
                }
                self.pressed = false;
            }
            if self.pressed {
                self.set_state(ButtonState::Pressed);
            }
            else {
                self.set_state(ButtonState::Hovered);
            }
        }
        else {
            self.pressed = false;
            self.set_state(ButtonState::Normal);
        }
    }
}

pub struct ButtonBuilder(ButtonStyle, String, Option<Signal>);

impl WidgetBuilder for ButtonBuilder {
    type Widget = Button;
    fn build_widget(self, gui: &mut Gui, node: Node) -> Button {
        let image = gui.add_child_widget(node, color_rect::ColorRect::new(self.0.normal_color));
        gui.set_style(image, Style {
            flex_grow: 1.0,
            padding: style::points_rect(self.0.padding),
            ..Default::default()
        });
        let mut text_widget = text::Text::new();
        text_widget.set_color(self.0.text_color);
        text_widget.set_text_all(self.0.font.clone(), self.0.text_size, self.1);
        text_widget.set_alignment(text::Align::Middle, text::Align::Middle);
        let text = gui.add_child_widget(image, text_widget);
        gui.set_style_fill_parent(text);
        Button {
            modification_queue: gui.modification_queue(),
            pressed_signal: self.2,
            style: self.0,
            pressed: false,
            toggled: false,
            image,
            text,
            state: ButtonState::Normal
        }
    }
}

pub struct ButtonToggledModifier {
    target: Node,
    toggled: bool,
}

impl ButtonToggledModifier {
    pub fn new(target: Node, toggled: bool) -> ButtonToggledModifier {
        ButtonToggledModifier { target, toggled }
    }
}

impl GuiModifier for ButtonToggledModifier {
    fn modify(&self, gui: &mut Gui) {
        let widget: &mut Button = gui.widget_mut(self.target).expect("ButtonToggledModifier target is not a Button");
        widget.set_toggled(self.toggled);
    }
}
