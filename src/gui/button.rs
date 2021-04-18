use std::any::Any;

use crate::color::Color;
use crate::geometry2d::Rect;
use super::{Gui, GuiNode, WidgetNode, Layout, Widget, DrawContext, GuiEventSystem, GuiInputEvent, GuiActionEvent, color_rect::ColorRect, text::{Text, Align}};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
    Disabled,
}

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
            hovered: Color::new(0.9, 0.9, 0.9, 1.0),
            pressed: Color::new(0.7, 0.7, 0.7, 1.0),
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
    color_rect: ColorRect,
    state: ButtonState,
    state_colors: ButtonStateColors,
}

impl Button {
    pub fn new(state_colors: ButtonStateColors) -> Button {
        Button { color_rect: ColorRect::new(state_colors.normal), state: ButtonState::Normal, state_colors }
    }
    fn set_state(&mut self, new_state: ButtonState) {
        self.state = new_state;
        self.color_rect.color = self.state_colors.get_color(new_state);
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
        self.color_rect.draw(context, rect);
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

pub struct ButtonBuilder {
    text: Option<String>,
    state_colors: ButtonStateColors,
}

impl ButtonBuilder {
    pub fn new() -> ButtonBuilder {
        ButtonBuilder { text: None, state_colors: ButtonStateColors::default() }
    }
    pub fn with_text(mut self, text: String) -> ButtonBuilder {
        self.text = Some(text);
        self
    }
    pub fn build(self, gui: &mut Gui, parent: GuiNode, layout: Layout) -> WidgetNode<Button> {
        let button = gui.add(parent, layout, Button::new(self.state_colors));
        if let Some(text_string) = self.text {
            let mut text = Text::new();
            text.set_text(text_string);
            text.set_alignment(Align::Middle, Align::Middle);
            gui.add(button.into(), Layout::fill_parent(0), text);
        }
        button
    }
}
