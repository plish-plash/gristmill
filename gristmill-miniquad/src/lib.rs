mod texture;
pub mod window;

use std::{
    hash::Hash,
    path::Path,
    time::{Duration, Instant},
};

use gristmill::{
    color::Color,
    logger,
    math::{vec2, Pos2, Rect, TSTransform, Vec2},
    scene2d::Instance,
    text::{TextBrush, TextPipeline},
    Buffer, DrawMetrics, Pipeline, Size,
};
use miniquad::*;

pub use miniquad::{KeyCode, MouseButton};
pub type InputEvent = gristmill::input::InputEvent<KeyCode, MouseButton>;
pub type Context = Box<dyn RenderingBackend>;

pub use texture::*;
pub use window::{WindowConfig, WindowSetup};

mod shader {
    use gristmill::math::Vec2;
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 vert_pos;

    attribute vec4 inst_rect;
    attribute vec4 inst_uv;
    attribute vec4 inst_color;

    varying lowp vec2 texcoord;
    varying lowp vec4 color;

    uniform vec4 transform;

    void main() {
        vec2 pos = (mix(inst_rect.xy, inst_rect.zw, vert_pos) + transform.xy) * transform.zw;
        gl_Position = vec4((pos - vec2(1.0, 1.0)) * vec2(1.0, -1.0), 0.0, 1.0);
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

    #[repr(C)]
    pub struct Uniforms {
        pub translate: Vec2,
        pub scale: Vec2,
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

pub struct InstanceBuffer {
    data: Vec<Instance>,
    data_changed: bool,
    buffer: Option<BufferId>,
    capacity: usize,
}

impl InstanceBuffer {
    const MINIMUM_SIZE: usize = 32;
    fn sync(&mut self, context: &mut Context) {
        if !self.data_changed {
            return;
        }
        if self.data.len() > self.capacity {
            self.capacity = self.data.len().next_power_of_two().max(Self::MINIMUM_SIZE);
            if let Some(buffer) = self.buffer {
                context.delete_buffer(buffer);
            }
            self.buffer = Some(context.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Stream,
                BufferSource::empty::<Instance>(self.capacity),
            ));
        }
        if let Some(buffer) = self.buffer {
            context.buffer_update(buffer, BufferSource::slice(&self.data));
        }
        self.data_changed = false;
    }
}
impl Buffer<Instance> for InstanceBuffer {
    fn new() -> Self {
        InstanceBuffer {
            data: Vec::new(),
            data_changed: false,
            buffer: None,
            capacity: 0,
        }
    }
    fn clear(&mut self) {
        self.data.clear();
        self.data_changed = true;
    }
    fn push(&mut self, value: Instance) {
        self.data.push(value);
        self.data_changed = true;
    }
}
impl Extend<Instance> for InstanceBuffer {
    fn extend<T: IntoIterator<Item = Instance>>(&mut self, iter: T) {
        self.data.extend(iter);
        self.data_changed = true;
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

impl gristmill::text::GlyphTexture<Pipeline2D> for GlyphTexture {
    fn resize(&mut self, context: &mut Context, size: Size) {
        context.delete_texture(self.0);
        *self = Self::new(context, size);
    }
    fn update(&mut self, context: &mut Context, min: [u32; 2], max: [u32; 2], data: &[u8]) {
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
    fn material(&self) -> Material {
        Material(Some(self.0))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Material(Option<TextureId>);

impl Material {
    pub const SOLID: Material = Material(None);
}

pub struct Pipeline2D {
    pipeline: miniquad::Pipeline,
    bindings: Bindings,
    solid_texture: TextureId,
    glyph_texture: GlyphTexture,
    draw_metrics: DrawMetrics,
    last_camera: TSTransform,
    last_clip: Option<Rect>,
}

impl Pipeline2D {
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
            vertex_buffers: vec![vertex_buffer, vertex_buffer],
            index_buffer,
            images: vec![none_texture],
        };

        Pipeline2D {
            pipeline,
            bindings,
            solid_texture: none_texture,
            glyph_texture,
            draw_metrics: DrawMetrics::new(),
            last_camera: TSTransform::IDENTITY,
            last_clip: None,
        }
    }

    pub fn bind(&mut self, context: &mut Context) {
        context.apply_pipeline(&self.pipeline);
        self.set_camera(context, &TSTransform::IDENTITY);
        self.last_camera = TSTransform::IDENTITY;
        self.last_clip = None;
    }
    fn set_camera(&mut self, context: &mut Context, camera: &TSTransform) {
        let (screen_width, screen_height) = window::screen_size();
        context.apply_uniforms(UniformsSource::table(&shader::Uniforms {
            translate: camera.translation / camera.scaling,
            scale: Vec2::new(2.0 / screen_width, 2.0 / screen_height) * camera.scaling,
        }));
    }
}

impl Pipeline for Pipeline2D {
    type Context = Context;
    type Material = Material;
    type Instance = Instance;
    type InstanceBuffer = InstanceBuffer;
    type Camera = TSTransform;
    fn transform(camera: &TSTransform, mut instance: Instance) -> Instance {
        instance.rect = *camera * instance.rect;
        instance.rect = Rect {
            min: instance.rect.min.round(),
            max: instance.rect.max.round(),
        };
        instance
    }
    fn draw(
        &mut self,
        context: &mut Context,
        camera: &TSTransform,
        material: &Material,
        instances: &mut InstanceBuffer,
    ) {
        if instances.data.is_empty() {
            return;
        }
        if *camera != self.last_camera {
            self.set_camera(context, camera);
            self.last_camera = *camera;
        }
        instances.sync(context);
        self.bindings.vertex_buffers[1] = instances.buffer.unwrap();
        self.bindings.images[0] = material.0.unwrap_or(self.solid_texture);
        context.apply_bindings(&self.bindings);
        context.draw(0, 6, instances.data.len() as i32);
        self.draw_metrics.draw_call();
    }
}
impl TextPipeline for Pipeline2D {
    #[allow(refining_impl_trait)]
    fn glyph_texture(&mut self) -> &mut GlyphTexture {
        &mut self.glyph_texture
    }
    fn set_clip(&mut self, context: &mut Self::Context, clip: Option<Rect>) {
        if clip != self.last_clip {
            self.last_clip = clip;
            let (screen_width, screen_height) = window::screen_size();
            let clip = clip.unwrap_or_else(|| {
                Rect::from_min_size(Pos2::ZERO, vec2(screen_width, screen_height))
            });
            context.apply_scissor_rect(
                clip.min.x as i32,
                (screen_height - clip.max.y) as i32,
                clip.width() as i32,
                clip.height() as i32,
            );
        }
    }
}

pub type Batcher2D<'a> = gristmill::Batcher<'a, Pipeline2D>;
pub type Sprite2D = gristmill::scene2d::sprite::Sprite<Material>;

pub struct Renderer2D {
    context: Context,
    pipeline: Pipeline2D,
    instances: InstanceBuffer,
}

impl Renderer2D {
    pub fn new(mut context: Context) -> Self {
        let pipeline = Pipeline2D::new(&mut context, None);
        Renderer2D {
            context,
            pipeline,
            instances: InstanceBuffer::new(),
        }
    }
    pub fn new_text<L>(mut context: Context, text_brush: &TextBrush<Pipeline2D, L>) -> Self
    where
        L: Clone + Eq + Hash + 'static,
    {
        let pipeline = Pipeline2D::new(&mut context, Some(text_brush.glyph_texture_size()));
        Renderer2D {
            context,
            pipeline,
            instances: InstanceBuffer::new(),
        }
    }
    pub fn context(&mut self) -> &mut Context {
        &mut self.context
    }
    pub fn begin_render(&mut self, background_color: Color) {
        self.context.begin_default_pass(PassAction::clear_color(
            background_color.r,
            background_color.g,
            background_color.b,
            background_color.a,
        ));
    }
    pub fn end_render(&mut self) -> DrawMetrics {
        self.context.end_render_pass();
        self.context.commit_frame();
        self.pipeline.draw_metrics.end_render()
    }
    pub fn bind_pipeline(&mut self) -> Batcher2D {
        self.pipeline.bind(&mut self.context);
        Batcher2D::new(&mut self.pipeline, &mut self.context, &mut self.instances)
    }
    pub fn process_text<L>(&mut self, text_brush: &mut TextBrush<Pipeline2D, L>)
    where
        L: Clone + Eq + Hash + 'static,
    {
        text_brush.process(&mut self.pipeline, &mut self.context);
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
        logger::set_message(format!("Working: {:.1}% | {}", working, metrics,));
    }

    // fn quit_requested_event(&mut self) {
    //     if !self.game_state.request_quit() {
    //         miniquad::window::cancel_quit();
    //     }
    // }
    fn resize_event(&mut self, width: f32, height: f32) {
        window::on_resize(width, height);
        self.game.resize(vec2(width, height));
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.game.input(InputEvent::MouseMotion {
            position: Pos2::new(x, y),
        });
    }
    fn raw_mouse_motion(&mut self, dx: f32, dy: f32) {
        self.game.input(InputEvent::RawMouseMotion {
            delta: vec2(dx, dy),
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

pub fn start<G: Game>(window_setup: WindowSetup, default_config: WindowConfig) {
    logger::init_logging(Some(Path::new("log.txt")));
    let config = window::load_config(window_setup, default_config);
    miniquad::start(config, move || Box::new(Stage::<G>::new()));
    window::save_config();
}
