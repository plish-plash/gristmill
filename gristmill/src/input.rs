use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use winit::event::{
    DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent,
};

use crate::asset::{Asset, AssetResult, AssetWrite, BufReader, BufWriter};
use crate::math::Vec2;

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
            InputState::Axis2(_) => panic!("axis2 input can't be bound to button action"),
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
            InputState::Axis2(_) => panic!("axis2 input can't be bound to axis1 action"),
        }
    }
    fn as_axis2(self) -> Vec2 {
        match self {
            InputState::Button(_) => panic!("button input can't be bound to axis2 action"),
            InputState::Axis1(v) => Vec2 { x: v, y: 0.0 },
            InputState::Axis2(v) => v,
        }
    }
}

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
    pub fn get(&self, key: &str) -> &ActionState {
        self.0.get(key).expect("action not bound")
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
pub struct InputBindings(Vec<(String, BindingEnum)>);

impl InputBindings {
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
        self.0.push((key.to_owned(), BindingEnum::Key(binding)));
    }
    pub fn add_key_axis1(&mut self, key: &str, binding: KeyAxis1Binding) {
        self.0
            .push((key.to_owned(), BindingEnum::KeyAxis1(binding)));
    }
    pub fn add_key_axis2(&mut self, key: &str, binding: KeyAxis2Binding) {
        self.0
            .push((key.to_owned(), BindingEnum::KeyAxis2(binding)));
    }
    pub fn add_mouse_button(&mut self, key: &str, binding: MouseButtonBinding) {
        self.0
            .push((key.to_owned(), BindingEnum::MouseButton(binding)));
    }
    pub fn add_mouse_motion(&mut self, key: &str, binding: MouseMotionBinding) {
        self.0
            .push((key.to_owned(), BindingEnum::MouseMotion(binding)));
    }
}

impl Asset for InputBindings {
    fn read_from(reader: BufReader) -> AssetResult<Self> {
        crate::asset::util::read_ron(reader)
    }
}

impl AssetWrite for InputBindings {
    fn write_to(value: &Self, writer: BufWriter) -> AssetResult<()> {
        crate::asset::util::write_ron(writer, value)
    }
}

pub struct InputSystem {
    bindings: InputBindings,
    actions: InputActions,
}

impl InputSystem {
    pub fn new(bindings: InputBindings) -> InputSystem {
        InputSystem {
            actions: bindings.create_actions(),
            bindings,
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
