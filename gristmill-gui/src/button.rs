use std::any::Any;
use std::sync::Arc;

use gristmill::color::Color;
use gristmill::geometry2d::Rect;
use super::{impl_class_field_fn, Gui, GuiNode, WidgetNode, Layout, Widget, DrawContext, GuiEventSystem, GuiInputEvent, GuiActionEvent, GuiNavigationEvent, GuiTexture, quad::Quad, text::{Text, Align}};

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
    hovered: bool,
    press_event: GuiActionEvent,
}

impl Button {
    pub fn new(state_colors: ButtonStateColors, press_event: GuiActionEvent) -> Button {
        Button { quad: Quad::new_color(state_colors.normal), state: ButtonState::Normal, state_colors, hovered: false, press_event }
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
    
    pub fn enabled(&self) -> bool { self.state != ButtonState::Disabled }
    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled && self.state == ButtonState::Disabled {
            self.set_state(if self.hovered { ButtonState::Hovered } else { ButtonState::Normal });
        }
        else if !enabled && self.state != ButtonState::Disabled {
            self.set_state(ButtonState::Disabled);
        }
    }
}

impl Widget for Button {
    fn as_any(&mut self) -> &mut dyn Any { self }
    fn draw(&mut self, context: &mut DrawContext, rect: Rect) {
        self.quad.draw(context, rect);
    }
    fn handle_input(&mut self, node: GuiNode, mut event_system: GuiEventSystem, input: GuiInputEvent) -> bool {
        match input {
            GuiInputEvent::CursorMoved(_) => event_system.fire_navigation(GuiNavigationEvent::Hover(node)),
            GuiInputEvent::PrimaryButton(down) => {
                if down {
                    self.transition_state(ButtonState::Hovered, ButtonState::Pressed);
                }
                else {
                    if self.state == ButtonState::Pressed {
                        self.set_state(ButtonState::Hovered);
                        event_system.fire_action(self.press_event.clone());
                    }
                }
            }
        }
        true
    }
    fn set_hovered(&mut self, hovered: bool) {
        self.hovered = hovered;
        if hovered {
            self.transition_state(ButtonState::Normal, ButtonState::Hovered);
        }
        else if self.state != ButtonState::Disabled {
            self.set_state(ButtonState::Normal);
        }
    }
    fn set_focused(&mut self, _focused: bool) {}
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

    pub fn instance_builder(&self) -> ButtonBuilder {
        ButtonBuilder { layout: Layout::default(), class: self, press_event: GuiActionEvent::Generic, text: None }
    }
}

pub struct ButtonBuilder<'a> {
    layout: Layout,
    class: &'a ButtonClass,
    press_event: GuiActionEvent,
    text: Option<String>,
}

impl<'a> ButtonBuilder<'a> {
    pub fn with_layout(mut self, layout: Layout) -> ButtonBuilder<'a> {
        self.layout = layout;
        self
    }
    pub fn with_press_event(mut self, press_event: GuiActionEvent) -> ButtonBuilder<'a> {
        self.press_event = press_event;
        self
    }
    pub fn with_text(mut self, text: String) -> ButtonBuilder<'a> {
        self.text = Some(text);
        self
    }
    pub fn build(self, gui: &mut Gui, parent: GuiNode) -> WidgetNode<Button> {
        let mut button_widget = Button::new(self.class.state_colors().cloned().unwrap_or_default(), self.press_event);
        if let Some(texture) = self.class.texture() {
            button_widget.set_texture(texture.clone());
        }
        let button = gui.add_widget(parent, self.layout, button_widget);
        if let Some(icon_texture) = self.class.icon() {
            gui.add_widget(button.into(), Layout::fill_parent(0), Quad::new_texture(icon_texture.clone()));
        }
        if let Some(text_string) = self.text {
            let mut text = Text::new(text_string);
            text.set_alignment(Align::Middle, Align::Middle);
            gui.add_widget(button.into(), Layout::fill_parent(0), text);
        }
        button
    }
}
