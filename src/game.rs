use std::{time::{Duration, Instant}, thread::sleep};

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use super::asset::load_asset;
use super::renderer::{RendererSetup, RendererLoader, Renderer, RenderLoop, RenderPass};
use super::input::{InputSystem, InputBindings};
use super::geometry2d::Size;

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
                Event::RedrawEventsCleared => {
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

pub trait Game {
    type RenderPass: RenderPass;
    fn load(&mut self, scene: &mut <Self::RenderPass as RenderPass>::Scene, renderer_setup: &mut RendererSetup) -> Self::RenderPass;
    fn update(&mut self, scene: &mut <Self::RenderPass as RenderPass>::Scene, window: &Window, input_system: &mut InputSystem, delta: f64) -> bool;
    fn update_renderer(&mut self, scene: &mut <Self::RenderPass as RenderPass>::Scene, render_pass: &mut Self::RenderPass, loader: &mut RendererLoader);
    fn resize(&mut self, scene: &mut <Self::RenderPass as RenderPass>::Scene, dimensions: Size);
}

pub fn run_game<G>(mut game: G, mut scene: <G::RenderPass as RenderPass>::Scene) -> ! where G: Game + 'static {
    let input_bindings = load_asset::<InputBindings>("controls").unwrap();
    let (mut renderer_setup, event_loop) = Renderer::create_window();
    let render_pass = game.load(&mut scene, &mut renderer_setup);
    RenderLoop::new(renderer_setup, render_pass, game, scene, InputSystem::new(input_bindings)).start(event_loop)
}
