mod texture;

use std::time::{Duration, Instant};

use gristmill::{
    color::Color,
    console,
    math::{Pos2, Vec2},
    scene2d::{CameraTransform, Instance},
    DrawMetrics, Renderer, Size,
};
use miniquad::*;

pub use miniquad::{conf::Conf as WindowConfig, KeyCode, MouseButton};
pub type InputEvent = gristmill::input::InputEvent<KeyCode, MouseButton>;
pub type Context = Box<dyn RenderingBackend>;

pub use texture::*;
pub mod window {
    pub use miniquad::window::{order_quit, request_quit, screen_size};
}

mod shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 vert_pos;

    attribute vec4 inst_rect;
    attribute vec4 inst_uv;
    attribute vec4 inst_color;

    varying mediump vec2 texcoord;
    varying lowp vec4 color;

    uniform vec4 transform;

    void main() {
        vec2 pos = mix(inst_rect.xy, inst_rect.zw, vert_pos);
        gl_Position = vec4((pos + transform.xy) * transform.zw, 0.0, 1.0);
        texcoord = mix(inst_uv.xy, inst_uv.zw, vert_pos);
        color = inst_color;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying mediump vec2 texcoord;
    varying lowp vec4 color;

    uniform sampler2D tex;

    void main() {
        gl_FragColor = texture2D(tex, texcoord) * color;
    }"#;

    #[repr(C)]
    pub struct Uniforms {
        pub transform: gristmill::scene2d::CameraTransform,
    }
    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![UniformDesc::new("transform", UniformType::Float4)],
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
    fn create_buffer(context: &mut Context, size: usize) -> BufferId {
        context.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Stream,
            BufferSource::empty::<Instance>(size),
        )
    }
    fn new(context: &mut Context) -> Self {
        InstanceBuffer {
            buffer: Self::create_buffer(context, Self::INITIAL_SIZE),
            size: Self::INITIAL_SIZE,
        }
    }
    fn set_data(&mut self, context: &mut Context, data: &[Instance]) {
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

pub struct GlyphTexture(TextureId);

impl GlyphTexture {
    fn new(context: &mut Context, size: Size) -> GlyphTexture {
        GlyphTexture(context.new_texture(
            TextureAccess::Static,
            TextureSource::Empty,
            TextureParams {
                kind: TextureKind::Texture2D,
                format: TextureFormat::RGBA8,
                width: size.width,
                height: size.height,
                ..Default::default()
            },
        ))
    }
}

impl gristmill::text::GlyphTexture for GlyphTexture {
    type Context = Context;
    type DrawParams = DrawParams;

    fn resize(&mut self, context: &mut Self::Context, size: Size) {
        context.delete_texture(self.0);
        *self = Self::new(context, size);
    }
    fn update(&mut self, context: &mut Self::Context, min: [u32; 2], max: [u32; 2], data: &[u8]) {
        let width = max[0] - min[0];
        let height = max[1] - min[1];
        let bytes: Vec<u8> = data.iter().flat_map(|x| [255, 255, 255, *x]).collect();
        context.texture_update_part(
            self.0,
            min[0] as i32,
            min[1] as i32,
            width as i32,
            height as i32,
            &bytes,
        );
    }
    fn draw_params(&self) -> Self::DrawParams {
        DrawParams(Some(self.0))
    }
}

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct DrawParams(Option<TextureId>);

impl DrawParams {
    pub fn texture(texture: &Texture) -> Self {
        DrawParams(Some(texture.id()))
    }
    pub fn texture_asset(texture: &TextureAsset) -> Self {
        DrawParams(Some(texture.id()))
    }
}

pub type Scene2D<L> = gristmill::Scene<L, DrawParams, Instance>;

pub struct Renderer2D {
    pipeline: Pipeline,
    instances: InstanceBuffer,
    bindings: Bindings,
    none_texture: TextureId,
    glyph_texture: GlyphTexture,
    draw_metrics: DrawMetrics,
}

impl Renderer2D {
    pub fn new(context: &mut Context, glyph_texture_size: Option<Size>) -> Self {
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
        let instances = InstanceBuffer::new(context);

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
        let none_texture = context.new_texture_from_rgba8(1, 1, &[255, 255, 255, 255]);
        let glyph_texture = glyph_texture_size
            .map(|size| GlyphTexture::new(context, size))
            .unwrap_or(GlyphTexture(none_texture));
        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer, instances.buffer],
            index_buffer,
            images: vec![none_texture],
        };

        Renderer2D {
            pipeline,
            instances,
            bindings,
            none_texture,
            glyph_texture,
            draw_metrics: DrawMetrics::new(),
        }
    }

    pub fn glyph_texture(&mut self) -> &mut GlyphTexture {
        &mut self.glyph_texture
    }

    pub fn begin_render(&mut self, context: &mut Context, background_color: Color) {
        context.begin_default_pass(PassAction::clear_color(
            background_color.r,
            background_color.g,
            background_color.b,
            background_color.a,
        ));
        context.apply_pipeline(&self.pipeline);
    }
    pub fn end_render(&mut self, context: &mut Context) -> DrawMetrics {
        context.end_render_pass();
        context.commit_frame();
        self.draw_metrics.end_render()
    }
    pub fn set_camera(&mut self, context: &mut Context, transform: CameraTransform) {
        context.apply_uniforms(UniformsSource::table(&shader::Uniforms { transform }));
    }
}

impl Renderer for Renderer2D {
    type Context = Context;
    type Params = DrawParams;
    type Instance = Instance;
    fn draw(
        &mut self,
        context: &mut Self::Context,
        params: &Self::Params,
        instances: &[Self::Instance],
    ) {
        self.instances.set_data(context, instances);
        self.bindings.vertex_buffers[1] = self.instances.buffer;
        self.bindings.images[0] = params.0.unwrap_or(self.none_texture);
        context.apply_bindings(&self.bindings);
        context.draw(0, 6, instances.len() as i32);
        self.draw_metrics.draw_call();
    }
}

pub trait Game: 'static {
    fn init(context: Context, screen_size: Vec2) -> Self;
    fn input(&mut self, event: InputEvent);
    fn update(&mut self, dt: f32);
    fn resize(&mut self, screen_size: Vec2);
    fn draw(&mut self) -> DrawMetrics;
}

struct Stage<G> {
    time: Instant,
    frame_time: Duration,
    sleep_time: Duration,
    game: G,
}

impl<G: Game> Stage<G> {
    fn new() -> Self {
        let context = miniquad::window::new_rendering_backend();
        let screen_size: Vec2 = window::screen_size().into();
        log::info!("Window size: {}x{}", screen_size.x, screen_size.y);
        Stage {
            time: Instant::now(),
            frame_time: Duration::from_secs_f32(1.0 / 60.0),
            sleep_time: Duration::ZERO,
            game: G::init(context, screen_size),
        }
    }
}

impl<G: Game> EventHandler for Stage<G> {
    fn update(&mut self) {
        let mut elapsed = self.time.elapsed();
        self.time = Instant::now();
        self.game.update(elapsed.as_secs_f32());
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
        let working = (1.0 - self.sleep_time.div_duration_f32(self.frame_time)) * 100.0;
        let metrics = self.game.draw();
        console::set_message(format!("Working: {:.1}% | {}", working, metrics,));
    }

    // fn quit_requested_event(&mut self) {
    //     if !self.game_state.request_quit() {
    //         miniquad::window::cancel_quit();
    //     }
    // }
    fn resize_event(&mut self, width: f32, height: f32) {
        self.game.resize(Vec2::new(width, height));
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.game.input(InputEvent::MouseMotion {
            position: Pos2::new(x, y),
        });
    }
    fn raw_mouse_motion(&mut self, dx: f32, dy: f32) {
        self.game.input(InputEvent::RawMouseMotion {
            delta: Vec2::new(dx, dy),
        });
    }
    fn mouse_wheel_event(&mut self, _x: f32, _y: f32) {
        // TODO
    }
    fn mouse_button_down_event(&mut self, button: MouseButton, _x: f32, _y: f32) {
        self.game.input(InputEvent::MouseButton {
            button,
            pressed: true,
        });
    }
    fn mouse_button_up_event(&mut self, button: MouseButton, _x: f32, _y: f32) {
        self.game.input(InputEvent::MouseButton {
            button,
            pressed: false,
        });
    }

    fn key_down_event(&mut self, keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        self.game.input(InputEvent::Key {
            key: keycode,
            pressed: true,
        });
    }
    fn key_up_event(&mut self, keycode: KeyCode, _keymods: KeyMods) {
        self.game.input(InputEvent::Key {
            key: keycode,
            pressed: false,
        });
    }
}

pub fn start<G: Game>(conf: WindowConfig) {
    console::init_logging();
    miniquad::start(conf, move || Box::new(Stage::<G>::new()));
}
