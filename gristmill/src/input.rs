use winit::event::{Event, VirtualKeyCode, MouseButton, WindowEvent, DeviceEvent, KeyboardInput, ElementState};
use winit::dpi::PhysicalPosition;
use serde::{Serialize, Deserialize};
use crate::geometry2d::Point;
use crate::impl_ron_asset;
use crate::event;

// -------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct Axis2 {
    pub x: f32,
    pub y: f32,
}

pub trait InputActions {
    fn end_frame(&mut self);
    fn set_action_state(&mut self, target: &str, state: ActionState);
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum InputState {
    Button(bool),
    Axis1(f32),
    Axis2(Axis2),
}

impl InputState {
    fn to_button(self) -> bool {
        match self {
            InputState::Button(b) => b,
            InputState::Axis1(v) => v.abs() >= 0.5,
            InputState::Axis2(_) => panic!("axis2 input can't be bound to button action"),
        }
    }
    fn to_axis1(self) -> f32 {
        match self {
            InputState::Button(b) => if b { 1.0 } else { 0.0 },
            InputState::Axis1(v) => v,
            InputState::Axis2(_) => panic!("axis2 input can't be bound to axis1 action"),
        }
    }
    fn to_axis2(self) -> Axis2 {
        match self {
            InputState::Button(_) => panic!("button input can't be bound to axis2 action"),
            InputState::Axis1(v) => Axis2 { x: v, y: 0.0 },
            InputState::Axis2(v) => v,
        }
    }
}

#[derive(Clone)]
pub struct ActionState {
    state: InputState,
    mouse_position: Option<Point>,
}

impl ActionState {
    fn new(state: InputState) -> ActionState {
        ActionState { state, mouse_position: None }
    }
    fn new_button() -> ActionState {
        Self::new(InputState::Button(false))
    }
    fn set_mouse_position(&mut self, mouse_position: PhysicalPosition<f64>) {
        let point = Point::nearest(mouse_position.x as f32, mouse_position.y as f32);
        self.mouse_position = Some(point);
    }
}

pub struct Action<T> {
    state: T,
    changed: bool,
}

impl<T> Action<T> {
    pub fn end_frame(&mut self) {
        self.changed = false;
    }
}
impl<T> Action<T> where T: Copy {
    pub fn get(&self) -> T { self.state }
}
impl<T> Action<T> where T: PartialEq {
    fn set_state_value(&mut self, state: T) {
        if self.state != state {
            self.state = state;
            self.changed = true;
        }
    }
}
impl<T> Default for Action<T> where T: Default {
    fn default() -> Action<T> {
        Action { state: T::default(), changed: false }
    }
}

impl Action<bool> {
    pub fn set_state(&mut self, state: ActionState) {
        self.set_state_value(state.state.to_button());
    }
    pub fn pressed(&self) -> bool {
        self.state && self.changed
    }
    pub fn released(&self) -> bool {
        !self.state && self.changed
    }
}
impl Action<f32> {
    pub fn set_state(&mut self, state: ActionState) {
        self.set_state_value(state.state.to_axis1());
    }
}
impl Action<Axis2> {
    pub fn set_state(&mut self, state: ActionState) {
        self.set_state_value(state.state.to_axis2());
    }
}

#[derive(Default)]
pub struct CursorAction {
    button: Action<bool>,
    position: Point,
}

impl CursorAction {
    pub fn get(&self) -> bool { self.button.get() }
    pub fn pressed(&self) -> bool { self.button.pressed() }
    pub fn released(&self) -> bool { self.button.released() }
    pub fn position(&self) -> Point { self.position }

    pub fn end_frame(&mut self) {
        self.button.end_frame();
    }
    pub fn set_state(&mut self, state: ActionState) {
        self.position = state.mouse_position.expect("only mouse buttons can be bound to CursorAction");
        self.button.set_state(state);
    }
}

// -------------------------------------------------------------------------------------------------

trait Binding {
    fn event(&mut self, event: &Event<()>) -> bool;
    fn state(&self) -> ActionState;
}

#[derive(Serialize, Deserialize)]
pub struct KeyboardBinding {
    key: VirtualKeyCode,
    #[serde(skip)]
    pressed: bool,
}

impl Binding for KeyboardBinding {
    fn event(&mut self, event: &Event<()>) -> bool {
        if let Event::WindowEvent { event: WindowEvent::KeyboardInput { input: KeyboardInput { state, virtual_keycode, .. }, .. }, .. } = event {
            if *virtual_keycode == Some(self.key) {
                self.pressed = *state == ElementState::Pressed;
                return true;
            }
        }
        false
    }
    fn state(&self) -> ActionState {
        ActionState::new(InputState::Button(self.pressed))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum CompositeDirection {
    Up, Down, Left, Right,
}

impl CompositeDirection {
    fn to_index(self) -> usize {
        match self {
            CompositeDirection::Up => 0,
            CompositeDirection::Down => 1,
            CompositeDirection::Left => 2,
            CompositeDirection::Right => 3,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct KeyboardCompositeBinding {
    directions: [KeyboardBinding; 4],
}

impl Binding for KeyboardCompositeBinding {
    fn event(&mut self, event: &Event<()>) -> bool {
        let mut changed = false;
        for binding in self.directions.iter_mut() {
            changed |= binding.event(event);
        }
        changed
    }
    fn state(&self) -> ActionState {
        let mut x = 0.;
        let mut y = 0.;
        if self.directions[CompositeDirection::Up.to_index()].pressed { y += 1.0; }
        if self.directions[CompositeDirection::Down.to_index()].pressed { y -= 1.0; }
        if self.directions[CompositeDirection::Left.to_index()].pressed { x -= 1.0; }
        if self.directions[CompositeDirection::Right.to_index()].pressed { x += 1.0; }
        ActionState::new(InputState::Axis2(Axis2 { x, y }))
    }
}

#[derive(Serialize, Deserialize)]
pub struct MouseButtonBinding {
    button: MouseButton,
    #[serde(skip, default="ActionState::new_button")]
    state: ActionState,
}

impl Binding for MouseButtonBinding {
    fn event(&mut self, event: &Event<()>) -> bool {
        if let Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } = event {
            self.state.set_mouse_position(*position);
            return true;
        }
        else if let Event::WindowEvent { event: WindowEvent::MouseInput { state, button, .. }, .. } = event {
            if *button == self.button {
                self.state.state = InputState::Button(*state == ElementState::Pressed);
                return true;
            }
        }
        false
    }
    fn state(&self) -> ActionState {
        self.state.clone()
    }
}

#[derive(Serialize, Deserialize)]
pub struct MouseMotionBinding {
    sensitivity: f32,
    #[serde(skip)]
    motion: Axis2,
}

impl Binding for MouseMotionBinding {
    fn event(&mut self, event: &Event<()>) -> bool {
        if let Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } = event {
            self.motion.x += delta.0 as f32 * self.sensitivity;
            self.motion.y += delta.1 as f32 * self.sensitivity;
        }
        false
    }
    fn state(&self) -> ActionState {
        ActionState::new(InputState::Axis2(self.motion))
    }
}

#[derive(Serialize, Deserialize)]
enum BindingEnum {
    // Button
    Keyboard(KeyboardBinding),
    MouseButton(MouseButtonBinding),
    // Axis2
    KeyboardComposite(KeyboardCompositeBinding),
    MouseMotion(MouseMotionBinding),
}

impl Binding for BindingEnum {
    fn event(&mut self, event: &Event<()>) -> bool {
        match self {
            BindingEnum::Keyboard(binding) => binding.event(event),
            BindingEnum::MouseButton(binding) => binding.event(event),
            BindingEnum::KeyboardComposite(binding) => binding.event(event),
            BindingEnum::MouseMotion(binding) => binding.event(event),
        }
    }
    fn state(&self) -> ActionState {
        match self {
            BindingEnum::Keyboard(binding) => binding.state(),
            BindingEnum::MouseButton(binding) => binding.state(),
            BindingEnum::KeyboardComposite(binding) => binding.state(),
            BindingEnum::MouseMotion(binding) => binding.state(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct InputBindings {
    bindings: Vec<(String, BindingEnum)>,
}

impl_ron_asset!(InputBindings, Config);

pub struct InputSystem {
    bindings: InputBindings,
    event_system: event::EventSystem<InputEvent>,
    has_dispatched: bool,
}

impl InputSystem {
    pub(crate) fn new(bindings: InputBindings) -> InputSystem {
        InputSystem { bindings, event_system: event::EventSystem::new(), has_dispatched: false }
    }
    pub(crate) fn start_frame(&mut self) {
        // MouseMotionBindings work differently than others. The values are accumulated over each frame, then reset.
        for (index, (_, binding)) in self.bindings.bindings.iter_mut().enumerate() {
            if let BindingEnum::MouseMotion(binding) = binding {
                self.event_system.fire_event(InputEvent::new(index));
                binding.motion = Axis2::default();
            }
        }
    }
    pub fn dispatch_queue<T: InputActions>(&mut self, actions: &mut T) {
        actions.end_frame();
        let bindings = &self.bindings;
        self.event_system.dispatch_queue(move |event| {
            let (target, binding) = &bindings.bindings[event.binding_index];
            actions.set_action_state(target, binding.state());
        });
        self.has_dispatched = true;
    }
    pub(crate) fn end_frame(&mut self) {
        if self.has_dispatched {
            self.has_dispatched = false;
        }
        else {
            self.event_system.discard_queue();
        }
    }
    pub(crate) fn input_event(&mut self, event: Event<()>) {
        // TODO do some matching so we're not checking every event against every binding
        for (index, (_, binding)) in self.bindings.bindings.iter_mut().enumerate() {
            if binding.event(&event) {
                self.event_system.fire_event(InputEvent::new(index));
            }
        }
    }
}

pub struct InputEvent {
    binding_index: usize
}

impl InputEvent {
    fn new(binding_index: usize) -> InputEvent {
        InputEvent { binding_index }
    }
}

impl event::Event for InputEvent {}
