use crate::{
    widget::{Image, StyleQuery, StyleValues, Text, TextAlign, Widget, WidgetInput},
    Gui, GuiDraw, GuiLayout, GuiNodeObj, WidgetBehavior,
};
use gristmill::{
    color::{IntoColor, LinLumaa},
    geom2d::{Rect, Size},
    Color,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Disabled,
    Normal,
    Hovered,
    Pressed,
}

impl Default for ButtonState {
    fn default() -> Self {
        ButtonState::Disabled
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ButtonColors {
    disabled: Color,
    normal: Color,
    hovered: Color,
    pressed: Color,
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            disabled: LinLumaa::new(0.75, 0.5).into_color(),
            normal: LinLumaa::new(0.75, 1.0).into_color(),
            hovered: LinLumaa::new(0.8, 1.0).into_color(),
            pressed: LinLumaa::new(0.9, 1.0).into_color(),
        }
    }
}

impl ButtonColors {
    fn get(&self, state: ButtonState) -> Color {
        match state {
            ButtonState::Disabled => self.disabled,
            ButtonState::Normal => self.normal,
            ButtonState::Hovered => self.hovered,
            ButtonState::Pressed => self.pressed,
        }
    }
}

#[derive(Clone, Default)]
struct ButtonBehaviorState {
    button_state: ButtonState,
    interactable: bool,
    just_released: bool,
}

struct ButtonBehavior {
    node: GuiNodeObj,
    colors: ButtonColors,
    state: RwLock<ButtonBehaviorState>,
}

impl WidgetBehavior for ButtonBehavior {
    fn update(&self, input: WidgetInput) {
        let mut cur_state = self.state.read().unwrap().clone();
        let button_state = if cur_state.interactable {
            if input.pointer_over == Some(self.node.key()) {
                if input.state.pressed() {
                    ButtonState::Pressed
                } else {
                    ButtonState::Hovered
                }
            } else {
                ButtonState::Normal
            }
        } else {
            ButtonState::Disabled
        };
        cur_state.interactable = false;
        if button_state != cur_state.button_state {
            cur_state.just_released = cur_state.button_state == ButtonState::Pressed
                && button_state == ButtonState::Hovered;
            cur_state.button_state = button_state;
            if let GuiDraw::Rect(_, color) = &mut self.node.write().draw {
                *color = self.colors.get(button_state);
            }
        } else {
            cur_state.just_released = false;
        }
        *self.state.write().unwrap() = cur_state;
    }
}

pub struct Button {
    node: GuiNodeObj,
    behavior: Arc<ButtonBehavior>,
    label: Text,
}

impl Button {
    pub fn interact(&self) -> bool {
        let mut write_guard = self.behavior.state.write().unwrap();
        write_guard.interactable = true;
        write_guard.just_released
    }
    pub fn state(&self) -> ButtonState {
        self.behavior.state.read().unwrap().button_state
    }
    pub fn set_label_string<S>(&self, text: S)
    where
        S: Into<String>,
    {
        self.label.set_text_string(text);
    }

    pub(crate) fn default_style() -> StyleValues {
        let mut style = StyleValues::new();
        style.set("size", Size::new(128, 32));
        style
    }
}

impl Widget for Button {
    fn class_name() -> &'static str {
        "Button"
    }
    fn new(gui: &mut Gui, parent: GuiNodeObj) -> Self {
        let node = Image::new(gui, parent).node().clone();
        let label = Text::new(gui, node.clone());
        label.set_layout(GuiLayout::fill());
        label.set_align(TextAlign::Middle);

        let behavior = Arc::new(ButtonBehavior {
            node: node.clone(),
            colors: ButtonColors::default(),
            state: RwLock::default(),
        });
        gui.register_behavior(behavior.clone());
        Button {
            node,
            behavior,
            label,
        }
    }
    fn apply_style(&mut self, style: StyleQuery) {
        self.node.write().layout =
            GuiLayout::Child(Rect::from_size(style.get("size", Size::new(128, 32))));
    }
    fn node(&self) -> &GuiNodeObj {
        &self.node
    }
}
