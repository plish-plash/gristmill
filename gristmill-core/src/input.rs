use crate::{
    asset::{self, AssetError},
    math::Vec2,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use winit::event::{
    DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent,
};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum InputState {
    Button(bool),
    Axis1(f32),
    Axis2(Vec2),
}

impl InputState {
    fn as_button(self) -> bool {
        match self {
            InputState::Button(b) => b,
            InputState::Axis1(v) => v.abs() >= 0.5,
            InputState::Axis2(_) => {
                log::error!("Axis2 input can't be bound to Button action.");
                false
            }
        }
    }
    fn as_axis1(self) -> f32 {
        match self {
            InputState::Button(b) => {
                if b {
                    1.0
                } else {
                    0.0
                }
            }
            InputState::Axis1(v) => v,
            InputState::Axis2(_) => {
                log::error!("Axis2 input can't be bound to Axis1 action.");
                0.0
            }
        }
    }
    fn as_axis2(self) -> Vec2 {
        match self {
            InputState::Button(_) => {
                log::error!("Button input can't be bound to Axis2 action.");
                Vec2::ZERO
            }
            InputState::Axis1(v) => Vec2 { x: v, y: 0.0 },
            InputState::Axis2(v) => v,
        }
    }
}

impl Default for InputState {
    fn default() -> Self {
        InputState::Button(false)
    }
}

#[derive(Copy, Clone, Default)]
pub struct ActionState {
    changed: bool,
    state: InputState,
    pointer: Option<Vec2>,
}

impl ActionState {
    fn new(state: InputState) -> Self {
        ActionState {
            changed: false,
            state,
            pointer: None,
        }
    }

    pub fn changed(&self) -> bool {
        self.changed
    }
    pub fn pointer(&self) -> Option<Vec2> {
        self.pointer
    }

    pub fn button_state(&self) -> bool {
        self.state.as_button()
    }
    pub fn axis1_state(&self) -> f32 {
        self.state.as_axis1()
    }
    pub fn axis2_state(&self) -> Vec2 {
        self.state.as_axis2()
    }

    pub fn pressed(&self) -> bool {
        self.button_state()
    }
    pub fn released(&self) -> bool {
        !self.button_state()
    }
    pub fn just_pressed(&self) -> bool {
        self.pressed() && self.changed
    }
    pub fn just_released(&self) -> bool {
        self.released() && self.changed
    }
}

#[derive(Default)]
pub struct InputActions(HashMap<String, ActionState>);

impl InputActions {
    fn end_frame(&mut self) {
        for (_, action) in self.0.iter_mut() {
            action.changed = false;
        }
    }
    fn set_state(&mut self, key: &str, state: InputState, pointer: Option<Vec2>) {
        if let Some(action) = self.0.get_mut(key) {
            action.pointer = pointer;
            if action.state != state {
                action.state = state;
                action.changed = true;
            }
        }
    }
    pub fn try_get(&self, key: &str) -> Option<&ActionState> {
        self.0.get(key)
    }
    pub fn get(&self, key: &str) -> ActionState {
        if let Some(state) = self.0.get(key) {
            *state
        } else {
            log::error!("Input action \"{}\" not bound.", key);
            ActionState::default()
        }
    }
}

trait Binding {
    fn event(&mut self, event: &Event<()>) -> bool;
    fn state(&self) -> InputState;
    fn pointer(&self) -> Option<Vec2> {
        None
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct KeyBinding {
    key: VirtualKeyCode,
    #[serde(skip)]
    pressed: bool,
}

impl KeyBinding {
    pub fn new(key: VirtualKeyCode) -> Self {
        KeyBinding {
            key,
            pressed: false,
        }
    }
}

impl Binding for KeyBinding {
    fn event(&mut self, event: &Event<()>) -> bool {
        if let Event::WindowEvent {
            event:
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state,
                            virtual_keycode,
                            ..
                        },
                    ..
                },
            ..
        } = event
        {
            if *virtual_keycode == Some(self.key) {
                self.pressed = *state == ElementState::Pressed;
                return true;
            }
        }
        false
    }
    fn state(&self) -> InputState {
        InputState::Button(self.pressed)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct KeyAxis1Binding {
    up: KeyBinding,
    down: KeyBinding,
}

impl KeyAxis1Binding {
    pub fn new(up: VirtualKeyCode, down: VirtualKeyCode) -> Self {
        KeyAxis1Binding {
            up: KeyBinding::new(up),
            down: KeyBinding::new(down),
        }
    }
}

impl Binding for KeyAxis1Binding {
    fn event(&mut self, event: &Event<()>) -> bool {
        let mut changed = false;
        changed |= self.up.event(event);
        changed |= self.down.event(event);
        changed
    }
    fn state(&self) -> InputState {
        let mut x = 0.;
        if self.up.pressed {
            x += 1.0;
        }
        if self.down.pressed {
            x -= 1.0;
        }
        InputState::Axis1(x)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct KeyAxis2Binding {
    up: KeyBinding,
    down: KeyBinding,
    left: KeyBinding,
    right: KeyBinding,
}

impl KeyAxis2Binding {
    pub fn new(
        up: VirtualKeyCode,
        down: VirtualKeyCode,
        left: VirtualKeyCode,
        right: VirtualKeyCode,
    ) -> Self {
        KeyAxis2Binding {
            up: KeyBinding::new(up),
            down: KeyBinding::new(down),
            left: KeyBinding::new(left),
            right: KeyBinding::new(right),
        }
    }
}

impl Binding for KeyAxis2Binding {
    fn event(&mut self, event: &Event<()>) -> bool {
        let mut changed = false;
        changed |= self.up.event(event);
        changed |= self.down.event(event);
        changed |= self.left.event(event);
        changed |= self.right.event(event);
        changed
    }
    fn state(&self) -> InputState {
        let mut x = 0.;
        let mut y = 0.;
        if self.up.pressed {
            y += 1.0;
        }
        if self.down.pressed {
            y -= 1.0;
        }
        if self.left.pressed {
            x -= 1.0;
        }
        if self.right.pressed {
            x += 1.0;
        }
        InputState::Axis2(Vec2 { x, y })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MouseButtonBinding {
    button: MouseButton,
    #[serde(skip)]
    state: (bool, [f32; 2]),
}

impl MouseButtonBinding {
    pub fn new(button: MouseButton) -> Self {
        MouseButtonBinding {
            button,
            state: Default::default(),
        }
    }
}

impl Binding for MouseButtonBinding {
    fn event(&mut self, event: &Event<()>) -> bool {
        if let Event::WindowEvent {
            event: WindowEvent::CursorMoved { position, .. },
            ..
        } = event
        {
            self.state.1 = position.cast::<f32>().into();
            return true;
        } else if let Event::WindowEvent {
            event: WindowEvent::MouseInput { state, button, .. },
            ..
        } = event
        {
            if *button == self.button {
                self.state.0 = *state == ElementState::Pressed;
                return true;
            }
        }
        false
    }
    fn state(&self) -> InputState {
        InputState::Button(self.state.0)
    }
    fn pointer(&self) -> Option<Vec2> {
        Some(self.state.1.into())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MouseMotionBinding {
    sensitivity: f32,
    #[serde(skip)]
    motion: Vec2,
}

impl MouseMotionBinding {
    pub fn new(sensitivity: f32) -> Self {
        MouseMotionBinding {
            sensitivity,
            motion: Vec2::ZERO,
        }
    }
}

impl Binding for MouseMotionBinding {
    fn event(&mut self, event: &Event<()>) -> bool {
        if let Event::DeviceEvent {
            event: DeviceEvent::MouseMotion { delta },
            ..
        } = event
        {
            self.motion.x += delta.0 as f32 * self.sensitivity;
            self.motion.y += delta.1 as f32 * self.sensitivity;
        }
        false
    }
    fn state(&self) -> InputState {
        InputState::Axis2(self.motion)
    }
}

#[derive(Clone, Serialize, Deserialize)]
enum BindingEnum {
    Key(KeyBinding),
    KeyAxis1(KeyAxis1Binding),
    KeyAxis2(KeyAxis2Binding),
    MouseButton(MouseButtonBinding),
    MouseMotion(MouseMotionBinding),
}

impl Binding for BindingEnum {
    fn event(&mut self, event: &Event<()>) -> bool {
        match self {
            BindingEnum::Key(binding) => binding.event(event),
            BindingEnum::KeyAxis1(binding) => binding.event(event),
            BindingEnum::KeyAxis2(binding) => binding.event(event),
            BindingEnum::MouseButton(binding) => binding.event(event),
            BindingEnum::MouseMotion(binding) => binding.event(event),
        }
    }
    fn state(&self) -> InputState {
        match self {
            BindingEnum::Key(binding) => binding.state(),
            BindingEnum::KeyAxis1(binding) => binding.state(),
            BindingEnum::KeyAxis2(binding) => binding.state(),
            BindingEnum::MouseButton(binding) => binding.state(),
            BindingEnum::MouseMotion(binding) => binding.state(),
        }
    }
    fn pointer(&self) -> Option<Vec2> {
        match self {
            BindingEnum::Key(binding) => binding.pointer(),
            BindingEnum::KeyAxis1(binding) => binding.pointer(),
            BindingEnum::KeyAxis2(binding) => binding.pointer(),
            BindingEnum::MouseButton(binding) => binding.pointer(),
            BindingEnum::MouseMotion(binding) => binding.pointer(),
        }
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InputBindings(HashMap<String, BindingEnum>);

impl InputBindings {
    pub fn load_config() -> Result<InputBindings, AssetError> {
        asset::load_yaml_file("config", "controls.yaml")
    }
    pub fn save_config(&self) -> Result<(), AssetError> {
        asset::save_yaml_file("config", "controls.yaml", self)
    }

    fn create_actions(&self) -> InputActions {
        InputActions(HashMap::from_iter(self.0.iter().map(|(key, binding)| {
            (key.clone(), ActionState::new(binding.state()))
        })))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn add_key(&mut self, key: &str, binding: KeyBinding) {
        self.0.insert(key.to_owned(), BindingEnum::Key(binding));
    }
    pub fn add_key_axis1(&mut self, key: &str, binding: KeyAxis1Binding) {
        self.0
            .insert(key.to_owned(), BindingEnum::KeyAxis1(binding));
    }
    pub fn add_key_axis2(&mut self, key: &str, binding: KeyAxis2Binding) {
        self.0
            .insert(key.to_owned(), BindingEnum::KeyAxis2(binding));
    }
    pub fn add_mouse_button(&mut self, key: &str, binding: MouseButtonBinding) {
        self.0
            .insert(key.to_owned(), BindingEnum::MouseButton(binding));
    }
    pub fn add_mouse_motion(&mut self, key: &str, binding: MouseMotionBinding) {
        self.0
            .insert(key.to_owned(), BindingEnum::MouseMotion(binding));
    }
}

pub struct InputSystem {
    bindings: InputBindings,
    actions: InputActions,
}

impl InputSystem {
    pub fn new(bindings: InputBindings) -> Self {
        InputSystem {
            actions: bindings.create_actions(),
            bindings,
        }
    }
    pub fn load_config() -> Self {
        match InputBindings::load_config() {
            Ok(bindings) => Self::new(bindings),
            Err(load_error) => {
                log::warn!("{}", load_error);
                type Key = VirtualKeyCode;
                let mut bindings = InputBindings::default();
                bindings.add_mouse_button("primary", MouseButtonBinding::new(MouseButton::Left));
                bindings.add_mouse_button("secondary", MouseButtonBinding::new(MouseButton::Right));
                bindings.add_mouse_motion("look", MouseMotionBinding::new(0.1));
                bindings.add_key("console", KeyBinding::new(Key::Grave));
                bindings.add_key("exit", KeyBinding::new(Key::Escape));
                bindings
                    .add_key_axis2("move", KeyAxis2Binding::new(Key::W, Key::S, Key::A, Key::D));
                bindings.add_key("jump", KeyBinding::new(Key::Space));
                bindings.add_key_axis1("fly", KeyAxis1Binding::new(Key::Space, Key::LShift));
                if load_error.io_kind() == Some(std::io::ErrorKind::NotFound) {
                    if let Err(save_error) = bindings.save_config() {
                        log::warn!("{}", save_error);
                    }
                }
                Self::new(bindings)
            }
        }
    }

    pub fn actions(&self) -> &InputActions {
        &self.actions
    }

    pub fn start_frame(&mut self) {
        // MouseMotionBindings work differently than others. The values are accumulated over each frame, then reset.
        for (key, binding) in self.bindings.0.iter_mut() {
            if let BindingEnum::MouseMotion(binding) = binding {
                self.actions
                    .set_state(key, binding.state(), binding.pointer());
                binding.motion = Vec2::ZERO;
            }
        }
    }
    pub fn end_frame(&mut self) {
        self.actions.end_frame();
    }

    pub fn input_event(&mut self, event: Event<()>) {
        for (key, binding) in self.bindings.0.iter_mut() {
            if binding.event(&event) {
                self.actions
                    .set_state(key, binding.state(), binding.pointer());
            }
        }
    }
}
