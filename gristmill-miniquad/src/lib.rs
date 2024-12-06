mod texture;

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use emath::{Pos2, Rect, Vec2};
use gristmill::{
    color::Color,
    gui::{GuiInput, GuiInputFrame, GuiRenderer},
    render2d::{Camera, Quad, QuadDrawQueue, ToQuad},
    text::{FontAsset, Text, TextDrawQueue},
    Dispatcher, Drawable, RenderQueue, Renderer,
};
use miniquad::*;

pub use miniquad::{conf::Conf as WindowConfig, KeyCode};
pub type InputEvent = gristmill::input::InputEvent<KeyCode>;
type RenderingContext = Box<dyn RenderingBackend>;

pub mod window {
    pub use miniquad::window::{order_quit, request_quit, screen_size};
}

mod shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    const vec2 FLIP_Y = vec2(1.0, -1.0);

    attribute vec2 vert_pos;

    attribute vec4 inst_rect;
    attribute vec4 inst_uv;
    attribute vec4 inst_color;

    varying lowp vec2 texcoord;
    varying lowp vec4 color;

    void main() {
        vec2 pos = mix(inst_rect.xy, inst_rect.zw, vert_pos);
        gl_Position = vec4((pos * 2.0 - 1.0) * FLIP_Y, 0.0, 1.0);
        texcoord = mix(inst_uv.xy, inst_uv.zw, vert_pos);
        color = inst_color;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec2 texcoord;
    varying lowp vec4 color;

    uniform sampler2D tex;

    void main() {
        gl_FragColor = texture2D(tex, texcoord) * color;
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: Vec::new(),
            },
        }
    }

    pub fn attributes() -> [VertexAttribute; 4] {
        [
            VertexAttribute::with_buffer("vert_pos", VertexFormat::Float2, 0),
            VertexAttribute::with_buffer("inst_rect", VertexFormat::Float4, 1),
            VertexAttribute::with_buffer("inst_uv", VertexFormat::Float4, 1),
            VertexAttribute::with_buffer("inst_color", VertexFormat::Float4, 1),
        ]
    }
}

struct InstanceBuffer {
    buffer: BufferId,
    size: usize,
}

impl InstanceBuffer {
    const INITIAL_SIZE: usize = 64;
    fn create_buffer(context: &mut RenderingContext, size: usize) -> BufferId {
        context.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Stream,
            BufferSource::empty::<Quad>(size),
        )
    }
    fn new(context: &mut RenderingContext) -> Self {
        InstanceBuffer {
            buffer: Self::create_buffer(context, Self::INITIAL_SIZE),
            size: Self::INITIAL_SIZE,
        }
    }
    fn set_data(&mut self, context: &mut RenderingContext, data: &[Quad]) {
        if data.len() > self.size {
            while data.len() > self.size {
                self.size *= 2;
            }
            context.delete_buffer(self.buffer);
            self.buffer = Self::create_buffer(context, self.size);
        }
        context.buffer_update(self.buffer, BufferSource::slice(data));
    }
}

pub struct DrawQuads(QuadDrawQueue, TextureId);

impl DrawQuads {
    fn new(context: &mut RenderingContext, dispatcher: Dispatcher) -> Self {
        let no_texture = context.new_texture_from_rgba8(1, 1, &[255, 255, 255, 255]);
        DrawQuads(QuadDrawQueue::new(dispatcher), no_texture)
    }
    pub fn queue<T: ToQuad>(&mut self, item: &T) {
        self.0.queue(item);
    }
    pub fn dispatch(&mut self) {
        self.0.dispatch();
    }
}

impl Drawable for DrawQuads {
    type Renderer = Renderer2D;
    fn draw_next(
        &mut self,
        renderer: &mut Self::Renderer,
    ) -> <Self::Renderer as Renderer>::DrawCall {
        let (texture, quads) = self.0.draw_next();
        renderer.instances.set_data(&mut renderer.context, quads);
        let texture = texture
            .and_then(|texture| {
                let asset: &texture::TextureAsset = texture
                    .handle
                    .downcast_ref()
                    .expect("invalid texture handle");
                asset.get()
            })
            .unwrap_or(self.1);
        (texture, quads.len())
    }
}

struct GlyphTexture<'a> {
    renderer: &'a mut Renderer2D,
    texture: TextureId,
}

impl<'a> GlyphTexture<'a> {
    fn create_texture(context: &mut RenderingContext, size: (u32, u32)) -> TextureId {
        context.new_texture(
            TextureAccess::Static,
            TextureSource::Empty,
            TextureParams {
                kind: TextureKind::Texture2D,
                format: TextureFormat::RGBA8,
                width: size.0,
                height: size.1,
                ..Default::default()
            },
        )
    }
}

impl<'a> gristmill::text::GlyphTexture for GlyphTexture<'a> {
    fn resize(&mut self, width: u32, height: u32) {
        self.renderer.context.delete_texture(self.texture);
        self.texture = Self::create_texture(&mut self.renderer.context, (width, height));
    }
    fn update(&mut self, min: [u32; 2], max: [u32; 2], data: &[u8]) {
        let width = max[0] - min[0];
        let height = max[1] - min[1];
        let bytes: Vec<u8> = data.iter().flat_map(|x| [255, 255, 255, *x]).collect();
        self.renderer.context.texture_update_part(
            self.texture,
            min[0] as i32,
            min[1] as i32,
            width as i32,
            height as i32,
            &bytes,
        );
    }
}

pub struct DrawText(TextDrawQueue, TextureId);

impl DrawText {
    fn new(context: &mut RenderingContext, dispatcher: Dispatcher, fonts: Vec<FontAsset>) -> Self {
        let text = TextDrawQueue::new(dispatcher, fonts);
        let texture = GlyphTexture::create_texture(context, text.glyph_texture_size());
        DrawText(text, texture)
    }
    pub fn queue(&mut self, text: &Text) {
        self.0.queue(text);
    }
    pub fn dispatch(&mut self) {
        self.0.dispatch();
    }
}

impl Drawable for DrawText {
    type Renderer = Renderer2D;
    fn draw_next(
        &mut self,
        renderer: &mut Self::Renderer,
    ) -> <Self::Renderer as Renderer>::DrawCall {
        let quads = self.0.draw_next();
        renderer.instances.set_data(&mut renderer.context, quads);
        (self.1, quads.len())
    }
}

pub struct Renderer2D {
    context: RenderingContext,
    pipeline: Pipeline,
    instances: InstanceBuffer,
    bindings: Bindings,
    draw_calls: usize,
}

impl Renderer2D {
    fn new() -> Self {
        let mut context = miniquad::window::new_rendering_backend();
        let vertices: [(f32, f32); 4] = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
        let vertex_buffer = context.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = context.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices),
        );
        let instances = InstanceBuffer::new(&mut context);

        let shader = context
            .new_shader(
                ShaderSource::Glsl {
                    vertex: shader::VERTEX,
                    fragment: shader::FRAGMENT,
                },
                shader::meta(),
            )
            .unwrap();
        let pipeline = context.new_pipeline(
            &[
                BufferLayout::default(),
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    ..Default::default()
                },
            ],
            &shader::attributes(),
            shader,
            PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
        );
        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer, instances.buffer],
            index_buffer,
            images: vec![TextureId::from_raw_id(RawId::OpenGl(0))],
        };

        Renderer2D {
            context,
            pipeline,
            instances,
            bindings,
            draw_calls: 0,
        }
    }

    fn draw_start(&mut self, background_color: Color) {
        self.draw_calls = 0;
        self.context.begin_default_pass(PassAction::clear_color(
            background_color.r,
            background_color.g,
            background_color.b,
            background_color.a,
        ));
        self.context.apply_pipeline(&self.pipeline);
    }
    fn draw_end(&mut self) {
        self.context.end_render_pass();
        self.context.commit_frame();
    }
}

impl Renderer for Renderer2D {
    type DrawCall = (TextureId, usize);
    fn draw(&mut self, draw_call: Self::DrawCall) {
        self.bindings.vertex_buffers[1] = self.instances.buffer;
        self.bindings.images[0] = draw_call.0;
        self.context.apply_bindings(&self.bindings);
        self.context.draw(0, 6, draw_call.1 as i32);
        self.draw_calls += 1;
    }
}

pub struct GameRenderer {
    renderer: Renderer2D,
    render_queue: Arc<RenderQueue>,
    pub quads: DrawQuads,
    pub text: DrawText,
    pub background_color: Color,
}

impl GameRenderer {
    fn new(fonts: Vec<FontAsset>) -> Self {
        let mut renderer = Renderer2D::new();
        let render_queue = RenderQueue::new();
        let quads = DrawQuads::new(&mut renderer.context, render_queue.get_dispatcher(0));
        let text = DrawText::new(&mut renderer.context, render_queue.get_dispatcher(1), fonts);
        GameRenderer {
            renderer,
            render_queue,
            quads,
            text,
            background_color: Color::BLACK,
        }
    }

    fn start(&mut self, camera: &Camera, screen_size: Vec2) {
        self.quads.0.start(camera.render_transform(screen_size));
        self.text.0.start(camera.screen_transform(screen_size));
    }
    fn draw(&mut self) -> usize {
        self.quads.dispatch();
        self.text.dispatch();
        texture::update_textures(&mut self.renderer.context);
        self.text.0.finish(&mut GlyphTexture {
            renderer: &mut self.renderer,
            texture: self.text.1,
        });

        self.renderer.draw_start(self.background_color);
        self.render_queue
            .draw(&mut self.renderer, vec![&mut self.quads, &mut self.text]);
        self.renderer.draw_end();
        self.renderer.draw_calls
    }
}

impl GuiRenderer for GameRenderer {
    fn quads(&mut self) -> &mut QuadDrawQueue {
        &mut self.quads.0
    }
    fn text(&mut self) -> &mut TextDrawQueue {
        &mut self.text.0
    }
}

pub trait GameInput: Default {
    fn event(&mut self, event: InputEvent);
    fn update(&mut self) {}
}

impl GameInput for () {
    fn event(&mut self, _event: InputEvent) {}
}

#[derive(Default)]
struct GameGuiInput<T> {
    game: T,
    gui: GuiInput,
}

impl<T: GameInput> GameGuiInput<T> {
    fn event(&mut self, event: InputEvent) {
        self.game.event(event.clone());
        self.gui.event(event);
    }
    fn mouse_event(&mut self, button: MouseButton, pressed: bool) {
        use gristmill::input;
        let button = match button {
            MouseButton::Left => input::MouseButton::Left,
            MouseButton::Middle => input::MouseButton::Middle,
            MouseButton::Right => input::MouseButton::Right,
            MouseButton::Unknown => return,
        };
        self.event(InputEvent::MouseButton { button, pressed });
    }
}

pub trait GameState {
    type Assets: GameAssets;
    type Input: GameInput;
    fn new(assets: Self::Assets) -> Self;
    fn update(&mut self, input: &mut Self::Input, frame_time: Duration);
    fn camera(&self) -> Camera;
    fn draw(&mut self, renderer: &mut GameRenderer, viewport: Rect, gui_input: GuiInputFrame);
    fn request_quit(&mut self) -> bool {
        true
    }
}

pub trait GameAssets: 'static {
    type GameState: GameState<Assets = Self>;
    fn window_config(&self) -> WindowConfig;
    fn fonts(&self) -> Vec<FontAsset>;
}

struct Stage<T: GameState> {
    time: Instant,
    frame_time: Duration,
    sleep_time: Duration,
    screen_size: Vec2,
    renderer: GameRenderer,
    input: GameGuiInput<T::Input>,
    game_state: T,
}

impl<T: GameState> Stage<T> {
    fn new(assets: T::Assets) -> Self {
        let renderer = GameRenderer::new(assets.fonts());
        let screen_size: Vec2 = window::screen_size().into();
        log::info!("Window size: {}x{}", screen_size.x, screen_size.y);
        Stage {
            time: Instant::now(),
            frame_time: Duration::from_secs_f32(1.0 / 60.0),
            sleep_time: Duration::ZERO,
            screen_size,
            renderer,
            input: Default::default(),
            game_state: T::new(assets),
        }
    }
}

impl<T: GameState> EventHandler for Stage<T> {
    fn update(&mut self) {
        let mut elapsed = self.time.elapsed();
        self.time = Instant::now();
        self.game_state.update(&mut self.input.game, elapsed);
        self.input.game.update();
        elapsed -= self.sleep_time;
        if elapsed < self.frame_time {
            // Limit framerate
            self.sleep_time = self.frame_time - elapsed;
            std::thread::sleep(self.sleep_time);
        } else {
            self.sleep_time = Duration::ZERO;
        }
    }
    fn draw(&mut self) {
        let camera = self.game_state.camera();
        self.renderer.start(&camera, self.screen_size);
        let screen_transform = camera.screen_transform(self.screen_size);
        self.game_state.draw(
            &mut self.renderer,
            camera.viewport(self.screen_size),
            self.input.gui.finish(&screen_transform),
        );
        let draw_calls = self.renderer.draw();

        let work_time = self.time.elapsed() - self.sleep_time;
        gristmill::console::set_message(format!(
            "Busy: {:.1}% | Draw calls: {}",
            work_time.div_duration_f32(self.frame_time) * 100.0,
            draw_calls
        ));
    }

    fn quit_requested_event(&mut self) {
        if !self.game_state.request_quit() {
            miniquad::window::cancel_quit();
        }
    }
    fn resize_event(&mut self, width: f32, height: f32) {
        self.screen_size = Vec2::new(width, height);
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.input.event(InputEvent::MouseMotion {
            position: Pos2::new(x, y),
        });
    }
    fn raw_mouse_motion(&mut self, dx: f32, dy: f32) {
        self.input.event(InputEvent::RawMouseMotion {
            delta: Vec2::new(dx, dy),
        });
    }
    fn mouse_wheel_event(&mut self, _x: f32, _y: f32) {
        // TODO
    }
    fn mouse_button_down_event(&mut self, button: MouseButton, _x: f32, _y: f32) {
        self.input.mouse_event(button, true);
    }
    fn mouse_button_up_event(&mut self, button: MouseButton, _x: f32, _y: f32) {
        self.input.mouse_event(button, false);
    }

    fn key_down_event(&mut self, keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        self.input.event(InputEvent::Key {
            key: keycode,
            pressed: true,
        });
    }
    fn key_up_event(&mut self, keycode: KeyCode, _keymods: KeyMods) {
        self.input.event(InputEvent::Key {
            key: keycode,
            pressed: false,
        });
    }
}

pub fn start<T, F>(f: F)
where
    T: GameAssets,
    F: FnOnce() -> T,
{
    gristmill::console::init_logging();
    let assets = f();
    let conf = assets.window_config();
    miniquad::start(conf, move || Box::new(Stage::<T::GameState>::new(assets)));
}
