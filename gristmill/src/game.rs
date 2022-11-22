use std::{
    thread,
    time::{Duration, Instant},
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use crate::{
    asset::{Asset, Resources},
    geom2d::Size,
    init_logging,
    input::{InputBindings, InputSystem},
    render::{RenderContext, RenderGameLoop},
};

// So users don't have to depend on winit
pub use winit::window::Window;

// -------------------------------------------------------------------------------------------------

const MINIMUM_FRAME_TIME: Duration = Duration::from_millis(15);

pub trait GameLoop: Sized + 'static {
    fn window(&self) -> &Window;
    fn update(&mut self, delta: f64) -> bool;
    fn event(&mut self, event: Event<()>);
    fn render(&mut self);

    fn start(mut self, event_loop: EventLoop<()>) -> ! {
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
                        self.window().request_redraw();
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

// -------------------------------------------------------------------------------------------------

pub trait Game: Sized + 'static {
    fn load(resources: Resources, context: &mut RenderContext) -> Self;
    fn resize(&mut self, _dimensions: Size) {}
    fn update(&mut self, window: &Window, input_system: &mut InputSystem, delta: f64);
    fn render(&mut self, context: &mut RenderContext);
}

pub fn run_game<G: Game>(resources: Resources) -> ! {
    init_logging();
    log::info!("Starting up...");
    let input_bindings = match InputBindings::read("controls") {
        Ok(bindings) => {
            log::debug!("Loaded controls ({} input bindings)", bindings.len());
            bindings
        }
        Err(error) => {
            log::error!("Failed to load controls: {}", error);
            Default::default()
        }
    };
    let event_loop = EventLoop::new();
    let mut context = RenderContext::create_window(&event_loop);
    let game = G::load(resources, &mut context);
    context.finish_setup();
    log::info!("Setup finished, entering main loop");
    RenderGameLoop::new(context, game, InputSystem::new(input_bindings)).start(event_loop)
}
