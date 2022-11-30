use crate::widget::StyleValue;
use crate::{
    widget::{Image, StyleQuery, StyleValues, Text, TextAlign, Widget, WidgetInput},
    Gui, GuiDraw, GuiLayout, GuiNodeObj, WidgetState,
};
use gristmill::{
    color::{IntoColor, LinLumaa},
    geom2d::{IRect, Size},
    render::texture::Texture,
    Color,
};
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
            disabled: LinLumaa::new(0.75, 0.5).into_color(),
            normal: LinLumaa::new(0.75, 1.0).into_color(),
            hovered: LinLumaa::new(0.85, 1.0).into_color(),
            pressed: LinLumaa::new(1.0, 1.0).into_color(),
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

struct ButtonWidgetState {
    node: GuiNodeObj,
    draw: ButtonDraw,
    state: ButtonState,
    interactable: bool,
    just_released: bool,
}

impl WidgetState for ButtonWidgetState {
    fn update(&mut self, input: WidgetInput) {
        let new_state = if self.interactable {
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
        self.interactable = false;
        if new_state != self.state {
            self.just_released =
                self.state == ButtonState::Pressed && new_state == ButtonState::Hovered;
            self.state = new_state;
            self.node.write().draw = self.draw.draw(new_state);
        } else {
            self.just_released = false;
        }
    }
}

pub struct Button {
    node: GuiNodeObj,
    state: Arc<RwLock<ButtonWidgetState>>,
    label: Text,
}

impl Button {
    pub fn interact(&self) -> bool {
        let mut write_guard = self.state.write().unwrap();
        write_guard.interactable = true;
        write_guard.just_released
    }
    pub fn state(&self) -> ButtonState {
        self.state.read().unwrap().state
    }
    pub fn set_label_string<S>(&self, text: S)
    where
        S: Into<String>,
    {
        self.label.set_text_string(text);
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
    fn class_name() -> &'static str {
        "button"
    }
    fn new(gui: &mut Gui, parent: GuiNodeObj) -> Self {
        let node = Image::new(gui, parent).node().clone();
        let label = Text::new(gui, node.clone());
        label.set_layout(GuiLayout::fill());
        label.set_align(TextAlign::Middle);

        let state = gui.register_widget_state(ButtonWidgetState {
            node: node.clone(),
            draw: ButtonDraw::default(),
            state: ButtonState::Disabled,
            interactable: false,
            just_released: false,
        });
        Button { node, state, label }
    }
    fn apply_style(&mut self, style: StyleQuery) {
        if let Some(texture) = style.get_texture("texture") {
            self.state.write().unwrap().draw = ButtonDraw::with_texture(texture);
        }
        let mut write_guard = self.node.write();
        write_guard.layout = GuiLayout::Child(IRect::from_size(
            style.get("size").unwrap_or(Size::new(128, 32)),
        ));
        write_guard.draw = self.state.read().unwrap().draw.draw(ButtonState::Disabled);
    }
    fn node(&self) -> &GuiNodeObj {
        &self.node
    }
}
