use crate::{
    widget::{
        Image, StyleQuery, StyleValue, StyleValues, Text, TextAlign, Widget, WidgetBehavior,
        WidgetInput, WidgetNode,
    },
    Gui, GuiDraw, GuiLayout, GuiNodeId, GuiNodeStorage,
};
use gristmill_core::{
    geom2d::{IRect, Size},
    Color,
};
use gristmill_render::Texture;
use std::{any::Any, cell::Cell, collections::HashMap, rc::Rc};

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
            hovered: Color::new_value(0.85),
            pressed: Color::new_value(1.0),
        }
    }
}

impl ButtonDraw {
    pub fn with_texture(texture: Texture) -> Self {
        ButtonDraw {
            texture: Some(texture),
            ..Default::default()
        }
    }

    fn draw(&self, state: ButtonState) -> GuiDraw {
        let color = match state {
            ButtonState::Disabled => self.disabled,
            ButtonState::Normal => self.normal,
            ButtonState::Hovered => self.hovered,
            ButtonState::Pressed => self.pressed,
        };
        GuiDraw::Rect(self.texture.clone(), color)
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

    pub(crate) fn default_style() -> StyleValues {
        let mut style = StyleValues::new();
        style.insert(
            "texture".to_owned(),
            crate::widget::style::make_empty_texture(),
        );
        style.insert(
            "size".to_owned(),
            StyleValue::try_from(Size::new(128, 32)).unwrap(),
        );
        style
    }
}

impl Widget for Button {
    fn type_name() -> &'static str {
        "Button"
    }
    fn new(gui: &mut Gui, parent: GuiNodeId, style: StyleQuery) -> Self {
        let draw = if let Some(texture) = style.get_texture(gui, "texture") {
            ButtonDraw::with_texture(texture)
        } else {
            ButtonDraw::default()
        };

        let image = Image::new(gui, parent, StyleQuery::default());
        image.set_layout(
            gui,
            GuiLayout::Child(IRect::from_size(
                style.get("size").unwrap_or(Size::new(128, 32)),
            )),
        );
        let node = image.node();
        gui.nodes.get_mut(node).unwrap().draw = draw.draw(ButtonState::Disabled);
        let label = Text::new(gui, node, style);
        label.set_layout(gui, GuiLayout::fill());
        label.set_align(gui, TextAlign::Middle);

        let behavior = gui.register_behavior(ButtonBehavior {
            node,
            draw,
            state: Cell::new(ButtonState::Disabled),
            interactable: Cell::new(false),
            just_released: Cell::new(false),
        });
        Button {
            node,
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
    fn unpack_extra_fields(&self, gui: &mut Gui, fields: &HashMap<String, StyleValue>) {
        if let Some(label) = fields.get("label").and_then(|value| value.as_str()) {
            self.set_label_string(gui, label);
        }
    }
}
