mod console;

pub use gristmill_core::*;
pub use gristmill_gui as gui;
pub use gristmill_render as render;

use crate::console::{ConsoleGame, LogRecord};
use gristmill_render::RenderContext;
use log::{Log, Metadata, Record};
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
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

pub fn run_game_with_console<G, F>(f: F) -> !
where
    G: Game,
    F: FnOnce(&mut RenderContext) -> G,
{
    let log_receiver = init_custom_logging();
    run_game(|context| {
        let game = f(context);
        ConsoleGame::new(context, log_receiver, game)
    })
}

struct CustomLogger(env_logger::Logger, mpsc::SyncSender<LogRecord>);

impl Log for CustomLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }
    fn log(&self, record: &Record) {
        self.0.log(record);
        self.1
            .try_send(LogRecord {
                level: record.level(),
                target: record.target().to_owned(),
                message: format!("{}", record.args()),
            })
            .ok();
    }
    fn flush(&self) {
        self.0.flush();
    }
}

fn env_logger_builder() -> env_logger::Builder {
    let default_log_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_log_level))
}
fn init_logging() {
    env_logger_builder().try_init().ok();
}
fn init_custom_logging() -> mpsc::Receiver<LogRecord> {
    let logger = env_logger_builder().build();
    let log_level = logger.filter();
    let (sender, receiver) = mpsc::sync_channel(100);
    log::set_boxed_logger(Box::new(CustomLogger(logger, sender))).ok();
    log::set_max_level(log_level);
    receiver
}
