pub use gristmill_core::*;
pub use gristmill_gui as gui;
pub use gristmill_macros::*;
pub use gristmill_render as render;

use gristmill_render::RenderContext;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{CursorGrabMode, Window},
};

pub trait Game: render::Renderable + 'static {
    fn input_system(&mut self) -> &mut input::InputSystem;
    fn update(&mut self, window: &mut GameWindow, delta: f64);
}

pub struct GameWindow<'a> {
    window: &'a Window,
    close: bool,
}

impl<'a> GameWindow<'a> {
    fn new(window: &'a Window) -> Self {
        GameWindow {
            window,
            close: false,
        }
    }
    pub fn close(&mut self) {
        self.close = true;
    }
    pub fn grab_cursor(&self) {
        self.window
            .set_cursor_grab(CursorGrabMode::Confined)
            .or_else(|_e| self.window.set_cursor_grab(CursorGrabMode::Locked))
            .unwrap();
        self.window.set_cursor_visible(false);
    }
    pub fn ungrab_cursor(&self) {
        self.window.set_cursor_grab(CursorGrabMode::None).unwrap();
        self.window.set_cursor_visible(true);
    }
}

struct GameLoop<G: Game> {
    game: G,
    context: RenderContext,
}

impl<G: Game> GameLoop<G> {
    fn update(&mut self, delta: f64) -> bool {
        self.game.input_system().start_frame();
        let mut window = GameWindow::new(self.context.window());
        self.game.update(&mut window, delta);
        self.game.input_system().end_frame();
        !window.close
    }
    fn event(&mut self, event: Event<()>) {
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                self.context.on_resize();
            }
            _ => self.game.input_system().input_event(event),
        }
    }
    fn render(&mut self) {
        self.context.render_game(&mut self.game);
    }

    fn start(self, event_loop: EventLoop<()>) -> ! {
        type InnerGameLoop<T> = game_loop::GameLoop<T, game_loop::Time, ()>;
        let mut game_loop = InnerGameLoop::new(self, 120, 0.1, ());
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
                    if !game_loop.next_frame(
                        |g| {
                            if !g.game.update(g.last_frame_time()) {
                                g.exit();
                            }
                        },
                        |g| g.game.render(),
                    ) {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                Event::MainEventsCleared => {
                    game_loop.game.context.window().request_redraw();
                }
                _ => {
                    game_loop.game.event(event);
                }
            }
        })
    }
}

fn init_logging() {
    let default_log_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_log_level))
        .try_init()
        .ok();
}

pub fn run_game<G, F>(f: F) -> !
where
    G: Game,
    F: FnOnce(&mut RenderContext) -> G,
{
    init_logging();
    log::info!("Starting up...");

    let event_loop = EventLoop::new();
    let mut context = RenderContext::create_window(&event_loop);
    let game = f(&mut context);
    context.finish_setup();

    log::info!("Setup finished, entering main loop.");
    GameLoop { game, context }.start(event_loop)
}
