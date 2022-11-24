use crate::{
    widget::{Image, StyleQuery, StyleValues, Text, TextAlign, Widget, WidgetInput},
    Gui, GuiDraw, GuiLayout, GuiNode, WidgetBehavior, WidgetObj,
};
use gristmill::{
    color::{IntoColor, LinLumaa},
    geom2d::{Rect, Size},
    Color, Obj,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Disabled,
    Normal,
    Hovered,
    Pressed,
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

struct ButtonBehavior {
    node: Obj<GuiNode>,
    colors: ButtonColors,
    state: ButtonState,
    interactable: bool,
    just_released: bool,
}

impl WidgetBehavior for ButtonBehavior {
    fn update(&mut self, input: WidgetInput) {
        let prev_state = self.state;
        self.state = if self.interactable {
            if input.pointer_over.as_ref() == Some(&self.node) {
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
        self.interactable = false;
        if self.state != prev_state {
            self.just_released =
                prev_state == ButtonState::Pressed && self.state == ButtonState::Hovered;
            if let GuiDraw::Rect(_, color) = &mut self.node.write().draw {
                *color = self.colors.get(self.state);
            }
        } else {
            self.just_released = false;
        }
    }
}

pub struct Button {
    node: Obj<GuiNode>,
    behavior: WidgetObj<ButtonBehavior>,
    label: Text,
}

impl Button {
    pub fn interact(&self) -> bool {
        let mut behavior = self.behavior.write();
        behavior.interactable = true;
        behavior.just_released
    }
    pub fn state(&self) -> ButtonState {
        self.behavior.read().state
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
    fn new(gui: &mut Gui, parent: Obj<GuiNode>) -> Self {
        let node = Image::new(gui, parent).node();
        let label = Text::new(gui, node.clone());
        label.set_layout(GuiLayout::fill());
        label.set_align(TextAlign::Middle);

        let behavior = gui.register_behavior(ButtonBehavior {
            node: node.clone(),
            colors: ButtonColors::default(),
            state: ButtonState::Disabled,
            interactable: false,
            just_released: false,
        });
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
    fn node(&self) -> Obj<GuiNode> {
        self.node.clone()
    }
}
