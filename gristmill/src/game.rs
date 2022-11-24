use std::{
    thread,
    time::{Duration, Instant},
};
use winit::window::Window;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use crate::{
    asset::AssetStorage,
    geom2d::Size,
    init_logging,
    input::{InputActions, InputBindings, InputSystem},
    render::RenderContext,
};

pub trait Game: Sized + 'static {
    fn load(config: AssetStorage, context: &mut RenderContext) -> Self;
    fn resize(&mut self, _dimensions: Size) {}
    fn update(&mut self, window: &mut GameWindow, input: &InputActions, delta: f64);
    fn render(&mut self, context: &mut RenderContext);
}

pub struct GameWindow<'a> {
    window: &'a Window,
    exit: bool,
}

impl<'a> GameWindow<'a> {
    fn new(window: &'a Window) -> Self {
        GameWindow {
            window,
            exit: false,
        }
    }
    pub fn exit(&mut self) {
        self.exit = true;
    }
}

struct GameLoop<G: Game> {
    context: RenderContext,
    game: G,
    input_system: InputSystem,
}

impl<G: Game> GameLoop<G> {
    pub fn new(context: RenderContext, game: G, input_system: InputSystem) -> Self {
        GameLoop {
            context,
            game,
            input_system,
        }
    }

    fn update(&mut self, delta: f64) -> bool {
        let mut window = GameWindow::new(self.context.window());
        self.input_system.start_frame();
        self.game
            .update(&mut window, self.input_system.actions(), delta);
        self.input_system.end_frame();
        !window.exit
    }
    fn event(&mut self, event: Event<()>) {
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                self.context.on_resize();
            }
            _ => self.input_system.input_event(event),
        }
    }
    fn render(&mut self) {
        self.context.render(&mut self.game);
    }

    fn start(mut self, event_loop: EventLoop<()>) -> ! {
        const MINIMUM_FRAME_TIME: Duration = Duration::from_millis(15);
        let mut last_frame_time = Instant::now();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::RedrawRequested(_) => {
                    self.render();
                }
                Event::MainEventsCleared => {
                    let mut current_frame_time = Instant::now();
                    let mut delta = current_frame_time.duration_since(last_frame_time);
                    if delta < MINIMUM_FRAME_TIME {
                        thread::sleep(MINIMUM_FRAME_TIME - delta);
                        current_frame_time = Instant::now();
                        delta = current_frame_time.duration_since(last_frame_time);
                    }
                    last_frame_time = current_frame_time;
                    if self.update(delta.as_secs_f64()) {
                        self.context.window().request_redraw();
                    } else {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                _ => {
                    self.event(event);
                }
            }
        })
    }
}

fn default_controls() -> InputBindings {
    use crate::input::*;
    use winit::event::MouseButton;
    use winit::event::VirtualKeyCode as Key;
    let mut controls = InputBindings::default();
    controls.add_mouse_button("primary", MouseButtonBinding::new(MouseButton::Left));
    controls.add_mouse_button("secondary", MouseButtonBinding::new(MouseButton::Right));
    controls.add_key("quit", KeyBinding::new(Key::Escape));
    controls.add_mouse_motion("look", MouseMotionBinding::new(0.1));
    controls.add_key_axis2("move", KeyAxis2Binding::new(Key::W, Key::S, Key::A, Key::D));
    controls.add_key("jump", KeyBinding::new(Key::Space));
    controls.add_key_axis1("fly", KeyAxis1Binding::new(Key::Space, Key::LShift));
    controls
}

pub fn run_game<G: Game>() -> ! {
    init_logging();
    log::info!("Starting up...");
    let event_loop = EventLoop::new();
    let mut context = RenderContext::create_window(&event_loop);

    let mut config = AssetStorage::config();
    let input_bindings = config.get_or_save("controls", default_controls).clone();
    let game = G::load(config, &mut context);
    context.finish_setup();

    log::info!("Setup finished, entering main loop");
    GameLoop::new(context, game, InputSystem::new(input_bindings)).start(event_loop)
}
