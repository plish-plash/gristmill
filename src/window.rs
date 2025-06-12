use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use silica_gui::{Hotkey, Point};
use silica_wgpu::{wgpu, AdapterFeatures, Context, Surface, SurfaceSize};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{KeyEvent, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::{
    error::{LoadGame, ResultExt},
    Game, GameInfo, GAME_INFO,
};

pub struct KeyboardEvent(pub KeyEvent);

impl crate::gui::KeyboardEvent for KeyboardEvent {
    fn to_hotkey(&self) -> Option<Hotkey> {
        None // TODO
    }
}

pub struct MouseButtonEvent(pub MouseButton, pub bool);

impl crate::gui::MouseButtonEvent for MouseButtonEvent {
    fn is_primary_button(&self) -> bool {
        self.0 == MouseButton::Left
    }
    fn is_pressed(&self) -> bool {
        self.1
    }
}

pub type InputEvent = crate::gui::InputEvent<KeyboardEvent, MouseButtonEvent>;

static RELOAD: AtomicBool = AtomicBool::new(false);
static EXIT: AtomicBool = AtomicBool::new(false);

pub fn reload() {
    log::debug!("Reload requested");
    RELOAD.store(true, Ordering::Relaxed);
}
pub fn exit() {
    log::debug!("Exit requested");
    EXIT.store(true, Ordering::Relaxed);
}

struct App<G: Game> {
    window: Option<Arc<Window>>,
    context: Context,
    surface: Surface,
    game: LoadGame<G>,
    min_frame_time: Duration,
    last_update: Instant,
}

impl<G: Game> App<G> {
    fn convert_size(size: PhysicalSize<u32>) -> SurfaceSize {
        SurfaceSize::new(size.width, size.height)
    }
    fn render(&mut self) {
        let frame = self.surface.acquire(&self.context);
        let view: wgpu::TextureView = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let clear_color = self.game.clear_color();
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear_color.r as f64,
                            g: clear_color.g as f64,
                            b: clear_color.b as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            self.game.render(&self.context, &mut pass);
        }
        self.context.queue.submit([encoder.finish()]);
        self.window.as_ref().unwrap().pre_present_notify();
        frame.present();
    }
}
impl<G: Game> ApplicationHandler for App<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_title = GAME_INFO.get().expect("game info not set").window_title;
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title(window_title))
                .unwrap(),
        );
        let size = Self::convert_size(window.inner_size());
        self.window = Some(window.clone());
        self.surface.resume(&self.context, window, size);
        if matches!(self.game, LoadGame::NotLoaded) {
            self.game.load(&self.context, self.surface.config().format);
        }
        self.last_update = Instant::now();
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.surface.suspend();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                let size = Self::convert_size(size);
                self.surface.resize(&self.context, size);
                self.game.resize(&self.context, size);
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - self.last_update).as_secs_f32();
                if dt > f32::EPSILON {
                    self.last_update = now;
                    self.game.update(dt);
                }
                self.render();
                event_loop.set_control_flow(ControlFlow::WaitUntil(now + self.min_frame_time));
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.game.input_event(InputEvent::MouseMotion(Point::new(
                    position.x as f32,
                    position.y as f32,
                )));
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.game
                    .input_event(InputEvent::MouseButton(MouseButtonEvent(
                        button,
                        state.is_pressed(),
                    )));
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.game
                    .input_event(InputEvent::Keyboard(KeyboardEvent(event)));
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if EXIT.load(Ordering::Relaxed) {
            event_loop.exit();
            return;
        }
        if let Some(window) = self.window.as_ref() {
            if RELOAD
                .compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                self.game.load(&self.context, self.surface.config().format);
                self.game
                    .resize(&self.context, Self::convert_size(window.inner_size()));
            }
            let now = Instant::now();
            if now - self.last_update >= self.min_frame_time {
                window.request_redraw();
            }
        }
    }
}

pub fn run_game<G: Game>(game_info: GameInfo) {
    crate::setup_environment(game_info);
    let context = Context::init(AdapterFeatures::default());
    let event_loop = EventLoop::new().unwrap_display();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        window: None,
        context,
        surface: Surface::new(),
        game: LoadGame::<G>::NotLoaded,
        min_frame_time: Duration::from_secs_f32(1.0 / 60.0),
        last_update: Instant::now(),
    };
    event_loop.run_app(&mut app).unwrap_display();
}
