use winit::event::{Event, VirtualKeyCode, MouseButton, WindowEvent, DeviceEvent, KeyboardInput, ElementState};
use winit::dpi::PhysicalPosition;
use serde::{Serialize, Deserialize};
use crate::gui::geometry::Point;

// ------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct Axis2 {
    pub x: f32,
    pub y: f32,
}

pub trait InputActions {
    fn end_frame(&mut self);
    fn set_action_state_button(&mut self, target: &str, state: ActionState<bool>);
    fn set_action_state_axis1(&mut self, target: &str, state: ActionState<f32>);
    fn set_action_state_axis2(&mut self, target: &str, state: ActionState<Axis2>);
}

#[derive(Clone)]
pub struct ActionState<T> {
    state: T,
    mouse_position: Option<Point>,
}

impl<T> ActionState<T> {
    fn from_state(state: T) -> ActionState<T> {
        ActionState { state, mouse_position: None }
    }
    fn set_mouse_position(&mut self, mouse_position: PhysicalPosition<f64>) {
        let point = Point::nearest(mouse_position.x as f32, mouse_position.y as f32);
        self.mouse_position = Some(point);
    }
}
impl<T> Default for ActionState<T> where T: Default {
    fn default() -> ActionState<T> {
        ActionState::from_state(T::default())
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
    pub fn set_state(&mut self, state: ActionState<T>) {
        if self.state != state.state {
            self.state = state.state;
            self.changed = true;
        }
    }
}
impl Action<bool> {
    pub fn pressed(&self) -> bool {
        self.state && self.changed
    }
    pub fn released(&self) -> bool {
        !self.state && self.changed
    }
}
impl<T> Default for Action<T> where T: Default {
    fn default() -> Action<T> {
        Action { state: T::default(), changed: false }
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
    pub fn set_state(&mut self, state: ActionState<bool>) {
        self.position = state.mouse_position.expect("only mouse buttons can be bound to CursorAction");
        self.button.set_state(state);
    }
}

// ------------------------------------------------------------------------------------------------

trait Binding<T> {
    fn event(&mut self, event: &Event<()>) -> bool;
    fn state(&self) -> ActionState<T>;
}

#[derive(Serialize, Deserialize)]
pub struct KeyboardBinding {
    key: VirtualKeyCode,
    #[serde(skip)]
    pressed: bool,
}

impl Binding<bool> for KeyboardBinding {
    fn event(&mut self, event: &Event<()>) -> bool {
        if let Event::WindowEvent { event: WindowEvent::KeyboardInput { input: KeyboardInput { state, virtual_keycode, .. }, .. }, .. } = event {
            if *virtual_keycode == Some(self.key) {
                self.pressed = *state == ElementState::Pressed;
                return true;
            }
        }
        false
    }
    fn state(&self) -> ActionState<bool> {
        ActionState::from_state(self.pressed)
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

impl Binding<Axis2> for KeyboardCompositeBinding {
    fn event(&mut self, event: &Event<()>) -> bool {
        let mut changed = false;
        for binding in self.directions.iter_mut() {
            changed |= binding.event(event);
        }
        changed
    }
    fn state(&self) -> ActionState<Axis2> {
        let mut x = 0.;
        let mut y = 0.;
        if self.directions[CompositeDirection::Up.to_index()].pressed { y += 1.0; }
        if self.directions[CompositeDirection::Down.to_index()].pressed { y -= 1.0; }
        if self.directions[CompositeDirection::Left.to_index()].pressed { x -= 1.0; }
        if self.directions[CompositeDirection::Right.to_index()].pressed { x += 1.0; }
        ActionState::from_state(Axis2 { x, y })
    }
}

#[derive(Serialize, Deserialize)]
pub struct MouseButtonBinding {
    button: MouseButton,
    #[serde(skip)]
    state: ActionState<bool>,
}

impl Binding<bool> for MouseButtonBinding {
    fn event(&mut self, event: &Event<()>) -> bool {
        if let Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } = event {
            self.state.set_mouse_position(*position);
            return true;
        }
        else if let Event::WindowEvent { event: WindowEvent::MouseInput { state, button, .. }, .. } = event {
            if *button == self.button {
                self.state.state = *state == ElementState::Pressed;
                return true;
            }
        }
        false
    }
    fn state(&self) -> ActionState<bool> {
        self.state.clone()
    }
}

#[derive(Serialize, Deserialize)]
pub struct MouseMotionBinding {
    sensitivity: f32,
    #[serde(skip)]
    motion: Axis2,
}

impl Binding<Axis2> for MouseMotionBinding {
    fn event(&mut self, event: &Event<()>) -> bool {
        if let Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } = event {
            self.motion.x += delta.0 as f32 * self.sensitivity;
            self.motion.y += delta.1 as f32 * self.sensitivity;
        }
        false
    }
    fn state(&self) -> ActionState<Axis2> {
        ActionState::from_state(self.motion)
    }
}

// TODO all of this is prime for macros

#[derive(Serialize, Deserialize)]
enum ButtonBinding {
    Keyboard(KeyboardBinding),
    MouseButton(MouseButtonBinding),
}

impl Binding<bool> for ButtonBinding {
    fn event(&mut self, event: &Event<()>) -> bool {
        match self {
            ButtonBinding::Keyboard(binding) => binding.event(event),
            ButtonBinding::MouseButton(binding) => binding.event(event),
        }
    }
    fn state(&self) -> ActionState<bool> {
        match self {
            ButtonBinding::Keyboard(binding) => binding.state(),
            ButtonBinding::MouseButton(binding) => binding.state(),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Axis1Binding {

}

impl Binding<f32> for Axis1Binding {
    fn event(&mut self, _event: &Event<()>) -> bool {
        unimplemented!();
    }
    fn state(&self) -> ActionState<f32> {
        unimplemented!();
    }
}

#[derive(Serialize, Deserialize)]
enum Axis2Binding {
    KeyboardComposite(KeyboardCompositeBinding),
    MouseMotion(MouseMotionBinding),
}

impl Binding<Axis2> for Axis2Binding {
    fn event(&mut self, event: &Event<()>) -> bool {
        match self {
            Axis2Binding::KeyboardComposite(binding) => binding.event(event),
            Axis2Binding::MouseMotion(binding) => binding.event(event),
        }
    }
    fn state(&self) -> ActionState<Axis2> {
        match self {
            Axis2Binding::KeyboardComposite(binding) => binding.state(),
            Axis2Binding::MouseMotion(binding) => binding.state(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Bindings {
    button: Vec<(String, ButtonBinding)>,
    axis1: Vec<(String, Axis1Binding)>,
    axis2: Vec<(String, Axis2Binding)>,
}

impl Bindings {
    fn event<T>(&mut self, actions: &mut T, event: Event<()>) where T: InputActions {
        for (target, binding) in self.button.iter_mut() {
            if binding.event(&event) {
                actions.set_action_state_button(target, binding.state());
            }
        }
        for (target, binding) in self.axis1.iter_mut() {
            if binding.event(&event) {
                actions.set_action_state_axis1(target, binding.state());
            }
        }
        for (target, binding) in self.axis2.iter_mut() {
            if binding.event(&event) {
                actions.set_action_state_axis2(target, binding.state());
            }
        }
    }
}

pub struct InputBindings<T> where T: InputActions {
    bindings: Bindings,
    actions: T,
}

impl<T> InputBindings<T> where T: InputActions {
    pub fn actions(&self) -> &T { &self.actions }

    pub fn start_frame(&mut self) {
        // MouseMotionBindings work differently than others. The values are accumulated over each frame, then reset.
        for (target, binding) in self.bindings.axis2.iter_mut() {
            if let Axis2Binding::MouseMotion(binding) = binding {
                self.actions.set_action_state_axis2(target, binding.state());
                binding.motion = Axis2::default();
            }
        }
    }
    pub fn end_frame(&mut self) {
        self.actions.end_frame();
    }
    pub fn event(&mut self, event: Event<()>) {
        self.bindings.event(&mut self.actions, event);
    }
}

impl<T> InputBindings<T> where T: InputActions + Default {
    pub fn load() -> std::io::Result<InputBindings<T>> {
        let bindings = crate::read_ron_file("controls.ron")?;
        Ok(InputBindings {
            bindings,
            actions: T::default(),
        })
    }
}
