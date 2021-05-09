use std::any::Any;
use std::sync::Arc;

use gristmill::color::Color;
use gristmill::geometry2d::Rect;
use super::{Gui, GuiNode, WidgetNode, Layout, Widget, DrawContext, GuiEventSystem, GuiInputEvent, GuiActionEvent, GuiTexture, quad::Quad, text::{Text, Align}};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
    Disabled,
}

#[derive(Clone)]
pub struct ButtonStateColors {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
    pub disabled: Color,
}

impl Default for ButtonStateColors {
    fn default() -> ButtonStateColors {
        ButtonStateColors {
            normal: Color::new(0.8, 0.8, 0.8, 1.0),
            hovered: Color::new(1.0, 1.0, 1.0, 1.0),
            pressed: Color::new(0.6, 0.6, 0.6, 1.0),
            disabled: Color::new(0.8, 0.8, 0.8, 0.5),
        }
    }
}

impl ButtonStateColors {
    fn get_color(&self, state: ButtonState) -> Color {
        match state {
            ButtonState::Normal => self.normal,
            ButtonState::Hovered => self.hovered,
            ButtonState::Pressed => self.pressed,
            ButtonState::Disabled => self.disabled,
        }
    }
}

pub struct Button {
    quad: Quad,
    state: ButtonState,
    state_colors: ButtonStateColors,
}

impl Button {
    pub fn new(state_colors: ButtonStateColors) -> Button {
        Button { quad: Quad::new_color(state_colors.normal), state: ButtonState::Normal, state_colors }
    }
    pub fn set_texture(&mut self, texture: GuiTexture) {
        self.quad.set_texture(texture);
    }
    fn set_state(&mut self, new_state: ButtonState) {
        self.state = new_state;
        self.quad.color = self.state_colors.get_color(new_state);
    }
    fn transition_state(&mut self, from_state: ButtonState, to_state: ButtonState) {
        if self.state == from_state {
            self.set_state(to_state);
        }
    }
}

impl Widget for Button {
    fn as_any(&mut self) -> &mut dyn Any { self }
    fn draw(&mut self, context: &mut DrawContext, rect: Rect) {
        self.quad.draw(context, rect);
    }
    fn handle_input(&mut self, node: GuiNode, event_system: &mut GuiEventSystem, input: GuiInputEvent) -> bool {
        match input {
            GuiInputEvent::CursorMoved(_) => event_system.fire_event(GuiActionEvent::Hover(node)),
            GuiInputEvent::PrimaryButton(down) => {
                if down {
                    self.transition_state(ButtonState::Hovered, ButtonState::Pressed);
                }
                else {
                    if self.state == ButtonState::Pressed {
                        self.set_state(ButtonState::Hovered);
                        event_system.fire_event(GuiActionEvent::Action(String::new()));
                    }
                }
            }
        }
        true
    }
    fn set_hovered(&mut self, hovered: bool) {
        if hovered {
            self.transition_state(ButtonState::Normal, ButtonState::Hovered);
        }
        else if self.state != ButtonState::Disabled {
            self.set_state(ButtonState::Normal);
        }
    }
    fn set_focused(&mut self, _focused: bool) {}
}

macro_rules! impl_class_field_fn {
    ($field:ident -> $field_type:ty) => {
        fn $field(&self) -> $field_type {
            if self.$field.is_some() {
                self.$field.as_ref()
            }
            else if let Some(parent) = self.parent.as_ref() {
                parent.$field()
            }
            else { None }
        }
    };
}

#[derive(Default)]
pub struct ButtonClass {
    parent: Option<Arc<ButtonClass>>,
    texture: Option<GuiTexture>,
    icon: Option<GuiTexture>,
    state_colors: Option<ButtonStateColors>,
}

impl ButtonClass {
    impl_class_field_fn!(texture -> Option<&GuiTexture>);
    impl_class_field_fn!(icon -> Option<&GuiTexture>);
    impl_class_field_fn!(state_colors -> Option<&ButtonStateColors>);
}

impl ButtonClass {
    pub fn new() -> ButtonClass {
        Default::default()
    }
    pub fn new_inherit(parent: Arc<ButtonClass>) -> ButtonClass {
        ButtonClass {
            parent: Some(parent),
            ..Default::default()
        }
    }

    pub fn set_texture(&mut self, texture: GuiTexture) {
        self.texture = Some(texture);
    }
    pub fn set_icon(&mut self, icon: GuiTexture) {
        self.icon = Some(icon);
    }

    pub fn instance(&self, gui: &mut Gui, parent: GuiNode, layout: Layout, text: Option<String>) -> WidgetNode<Button> {
        let mut button_widget = Button::new(self.state_colors().cloned().unwrap_or_default());
        if let Some(texture) = self.texture() {
            button_widget.set_texture(texture.clone());
        }
        let button = gui.add_widget(parent, layout, button_widget);
        if let Some(icon_texture) = self.icon() {
            gui.add_widget(button.into(), Layout::fill_parent(0), Quad::new_texture(icon_texture.clone()));
        }
        if let Some(text_string) = text {
            let mut text = Text::new(text_string);
            text.set_alignment(Align::Middle, Align::Middle);
            gui.add_widget(button.into(), Layout::fill_parent(0), text);
        }
        button
    }
}
