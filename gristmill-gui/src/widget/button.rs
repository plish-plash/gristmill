use crate::widget::{
    Image, ImageStyle, InputState, StyleValues, Text, TextAlign, TextStyle, Widget, WidgetType,
};
use crate::{Gui, GuiDraw, GuiLayout, GuiNode, GuiTexture, WidgetBehavior, WidgetObj};
use gristmill::color::{IntoColor, LinLumaa};
use gristmill::geom2d::Size;
use gristmill::{Color, Obj};
use serde::{Deserialize, Serialize};

pub struct ButtonStyle {
    pub background_colors: ButtonColors,
    pub label: TextStyle,
    pub size: Size,
}

impl ButtonStyle {
    fn from_style(_style: &StyleValues) -> ButtonStyle {
        // TODO
        ButtonStyle::default()
    }
}
impl Default for ButtonStyle {
    fn default() -> Self {
        let background_colors = ButtonColors {
            disabled: LinLumaa::new(0.75, 0.5).into_color(),
            normal: LinLumaa::new(0.75, 1.0).into_color(),
            hovered: LinLumaa::new(0.8, 1.0).into_color(),
            pressed: LinLumaa::new(0.9, 1.0).into_color(),
        };
        ButtonStyle {
            background_colors,
            label: TextStyle::default(),
            size: Size::new(128, 32),
        }
    }
}

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
    fn node(&self) -> Obj<GuiNode> {
        self.node.clone()
    }
    fn update(&mut self, state: InputState) {
        let prev_state = self.state;
        self.state = if self.interactable {
            if state.cursor_over {
                if state.input.primary().pressed() {
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
    pub fn create_with_button_style(
        gui: &mut Gui,
        parent: Obj<GuiNode>,
        style: ButtonStyle,
    ) -> Button {
        let node = Image::create_with_image_style(
            parent,
            ImageStyle {
                draw: GuiDraw::Rect(GuiTexture::default(), style.background_colors.disabled),
                size: style.size,
            },
        )
        .node();
        let behavior = gui.register_behavior(ButtonBehavior {
            node: node.clone(),
            colors: style.background_colors,
            state: ButtonState::Disabled,
            interactable: false,
            just_released: false,
        });
        let mut label = Text::create_with_text_style(node.clone(), style.label);
        label.set_layout(GuiLayout::fill());
        label.set_align(TextAlign::Middle);
        Button {
            node,
            behavior,
            label,
        }
    }

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
}

impl Widget for Button {
    fn widget_type() -> WidgetType {
        WidgetType::button()
    }
    fn create_with_style(gui: &mut Gui, parent: Obj<GuiNode>, style: &StyleValues) -> Button {
        Self::create_with_button_style(gui, parent, ButtonStyle::from_style(style))
    }
    fn node(&self) -> Obj<GuiNode> {
        self.node.clone()
    }
}
