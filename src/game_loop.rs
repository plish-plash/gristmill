use std::{time::{Duration, Instant}, thread::sleep};

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

// ------------------------------------------------------------------------------------------------

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
