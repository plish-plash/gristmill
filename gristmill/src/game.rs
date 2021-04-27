use std::{time::{Duration, Instant}, thread::sleep};

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use super::asset::load_asset;
use super::renderer::{RenderPassInfo, RenderContext, Renderer, RenderLoop};
use super::input::{InputSystem, InputBindings};
use super::geometry2d::Size;

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
                },
                Event::MainEventsCleared => {
                    let mut current_frame_time = Instant::now();
                    let mut delta = current_frame_time.duration_since(last_frame_time);
                    if delta < MINIMUM_FRAME_TIME {
                        sleep(MINIMUM_FRAME_TIME - delta);
                        current_frame_time = Instant::now();
                        delta = current_frame_time.duration_since(last_frame_time);
                    }
                    last_frame_time = current_frame_time;
                    if self.update(delta.as_secs_f64()) {
                        self.window().request_redraw();
                    }
                    else {
                        *control_flow = ControlFlow::Exit;
                    }
                },
                _ => {
                    self.event(event);
                }
            }
        })
    }
}

// -------------------------------------------------------------------------------------------------

pub trait Game: Sized + 'static {
    fn load(renderer: &mut Renderer) -> (Self, RenderPassInfo);
    fn resize(&mut self, dimensions: Size);
    fn update(&mut self, window: &Window, input_system: &mut InputSystem, delta: f64) -> bool;
    fn render(&mut self, renderer: &mut Renderer, context: &mut RenderContext);
}

pub fn run_game<G: Game>() -> ! {
    let input_bindings = load_asset::<InputBindings>("controls").unwrap();
    let (mut renderer, event_loop) = Renderer::create_window();
    let (game, render_pass) = G::load(&mut renderer);
    RenderLoop::new(renderer, game, render_pass, InputSystem::new(input_bindings)).start(event_loop)
}
