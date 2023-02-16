use crate::{
    widget::{
        Image, StyleValues, Text, Widget, WidgetBehavior, WidgetInput, WidgetNode, WidgetNodeExt,
        WidgetStyle,
    },
    Anchor, Gui, GuiNodeId, GuiNodeStorage, NodeDraw,
};
use gristmill_core::Color;
use gristmill_render::Texture;
use std::{any::Any, cell::Cell, rc::Rc};

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

impl ButtonState {
    pub fn is_active(self) -> bool {
        self == ButtonState::Hovered || self == ButtonState::Pressed
    }
}

struct ButtonDraw {
    texture: Option<Texture>,
    //animate_texture: bool,
    pub disabled: Color,
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
}

impl Default for ButtonDraw {
    fn default() -> Self {
        ButtonDraw {
            texture: None,
            disabled: Color::new(0.75, 0.75, 0.75, 0.5),
            normal: Color::new_value(0.75),
            hovered: Color::new_value(0.8),
            pressed: Color::new_value(0.9),
        }
    }
}

impl ButtonDraw {
    pub fn with_texture(texture: Texture) -> Self {
        ButtonDraw {
            texture: Some(texture),
            disabled: Color::new(1.0, 1.0, 1.0, 0.5),
            normal: Color::new_value(1.0),
            hovered: Color::new_value(0.9),
            pressed: Color::new_value(0.75),
        }
    }

    fn draw(&self, state: ButtonState) -> NodeDraw {
        let color = match state {
            ButtonState::Disabled => self.disabled,
            ButtonState::Normal => self.normal,
            ButtonState::Hovered => self.hovered,
            ButtonState::Pressed => self.pressed,
        };
        NodeDraw::Rect(self.texture.clone(), color)
    }
}

struct ButtonBehavior {
    node: GuiNodeId,
    draw: ButtonDraw,
    state: Cell<ButtonState>,
    interactable: Cell<bool>,
    just_released: Cell<bool>,
}

impl WidgetBehavior for ButtonBehavior {
    fn update(&self, nodes: &mut GuiNodeStorage, input: &WidgetInput) {
        let new_state = if self.interactable.get() {
            if input.pointer_over == Some(self.node) {
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
        self.interactable.set(false);
        if new_state != self.state.get() {
            self.just_released
                .set(self.state.get() == ButtonState::Pressed && new_state == ButtonState::Hovered);
            self.state.set(new_state);
            if let Some(node) = nodes.get_mut(self.node) {
                node.draw = self.draw.draw(new_state);
            }
        } else {
            self.just_released.set(false);
        }
    }
}

pub struct Button {
    node: GuiNodeId,
    label: Text,
    behavior: Rc<ButtonBehavior>,
}

impl Button {
    pub fn interact(&mut self) -> bool {
        self.behavior.interactable.set(true);
        self.behavior.just_released.get()
    }
    pub fn state(&self) -> ButtonState {
        self.behavior.state.get()
    }
    pub fn set_label_string<S>(&self, gui: &mut Gui, text: S)
    where
        S: Into<String>,
    {
        self.label.set_text_string(gui, text);
    }
}

impl Widget for Button {
    fn class_name() -> &'static str {
        "button"
    }
    fn new(gui: &mut Gui, parent: GuiNodeId, mut style: StyleValues) -> Self {
        let draw = if let Some(texture) = style.widget_value("texture", None) {
            ButtonDraw::with_texture(texture)
        } else {
            ButtonDraw::default()
        };
        let label_text = style.widget_value("label", String::new());

        let image = Image::new(gui, parent, StyleValues::new());
        let image_node = image.node_data(gui).unwrap();
        image_node.flags.pointer_opaque = true;
        image_node.layout = style.widget_layout();
        image_node.draw = draw.draw(ButtonState::Disabled);
        let label = Text::new(gui, image.node(), StyleValues::new());
        label.set_text_align(gui, (Anchor::Middle, Anchor::Middle), false);
        label.set_text_string(gui, label_text);

        let behavior = gui.register_behavior(ButtonBehavior {
            node: image.node(),
            draw,
            state: Cell::new(ButtonState::Disabled),
            interactable: Cell::new(false),
            just_released: Cell::new(false),
        });
        Button {
            node: image.node(),
            label,
            behavior,
        }
    }
}

impl WidgetNode for Button {
    fn as_any_box(self: Box<Self>) -> Box<dyn Any> {
        self
    }
    fn node(&self) -> GuiNodeId {
        self.node
    }
}
