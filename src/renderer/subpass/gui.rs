use rusttype::{PositionedGlyph, Scale, point};
use rusttype::gpu_cache::Cache;

use vulkano::buffer::{CpuAccessibleBuffer, ImmutableBuffer, BufferUsage, BufferAccess};
use vulkano::command_buffer::{DynamicState, AutoCommandBufferBuilder, SubpassContents};
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::descriptor::pipeline_layout::PipelineLayoutAbstract;
use vulkano::device::Device;
use vulkano::format::R8Unorm;
use vulkano::image::{AttachmentImage, ImageUsage};
use vulkano::image::view::ImageView;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::sampler::{Sampler, Filter, MipmapMode, SamplerAddressMode};
use vulkano::instance::QueueFamily;

use std::sync::{Arc, Weak};

use crate::color::{Color, encode_color};
use crate::renderer::{PipelineArc, SubpassSetup, subpass::{self, Pipeline, RenderSubpass}};
use crate::gui::{Gui, Node, geometry::{Rect, Point, Size}, font::fonts};

#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_position: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position, tex_position);

impl Vertex {
    fn new(position: [i32; 2], tex_position: [f32; 2]) -> Vertex {
        let position = [position[0] as f32, position[1] as f32];
        Vertex { position, tex_position }
    }
}

mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        src: "
            #version 450

            layout(push_constant) uniform PushConstants {
                vec2 screen_size;
                vec2 position;
                vec2 size;
                vec4 color;
            } constants;

            layout(location = 0) in vec2 position;
            layout(location = 1) in vec2 tex_position;
            layout(location = 0) out vec2 v_tex_position;
            layout(location = 1) out vec4 v_color;

            void main() {
                vec2 normalized_position = (constants.position + (position * constants.size)) / constants.screen_size;
                gl_Position = vec4((normalized_position - 0.5) * 2.0, 0.0, 1.0);
                v_tex_position = tex_position;
                v_color = constants.color;
            }
        "
    }
}

mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        src: "
            #version 450

            layout(location = 0) in vec2 v_tex_position;
            layout(location = 1) in vec4 v_color;
            layout(location = 0) out vec4 f_color;

            layout(set = 0, binding = 0) uniform sampler2D tex;

            void main() {
                f_color = v_color;
                if (v_tex_position.x >= 0) {
                    f_color *= texture(tex, v_tex_position)[0];
                }
            }
        "
    }
}

pub struct GuiPipeline {
    pipeline: PipelineArc,
    descriptor_set: Arc<dyn DescriptorSet + Send + Sync>,
    square_vertex_buffer: Arc<ImmutableBuffer<[Vertex]>>,
    text_cache_image: Arc<AttachmentImage<R8Unorm>>,
}

impl GuiPipeline {
    fn new(subpass_setup: &mut SubpassSetup) -> GuiPipeline {
        let vs = vs::Shader::load(subpass_setup.device()).unwrap();
        let fs = fs::Shader::load(subpass_setup.device()).unwrap();

        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .blend_alpha_blending()
                .render_pass(subpass_setup.subpass())
                .build(subpass_setup.device())
                .unwrap()
        );

        let (square_vertex_buffer, setup_future) = ImmutableBuffer::from_iter(
            vec![
                Vertex::new([0, 1], [-1., -1.]),
                Vertex::new([0, 0], [-1., -1.]),
                Vertex::new([1, 0], [-1., -1.]),

                Vertex::new([1, 0], [-1., -1.]),
                Vertex::new([1, 1], [-1., -1.]),
                Vertex::new([0, 1], [-1., -1.]),
            ].into_iter(),
            BufferUsage::vertex_buffer(),
            subpass_setup.queue(),
        ).unwrap();
        subpass_setup.queue_join(setup_future);

        // Note: because this is an AttachmentImage, the color_attachment usage is added implicitly, which is unneeded.
        // However, Vulkano doesn't provide a "DeviceLocalImage", so this is what we've got.
        let text_cache_image = AttachmentImage::with_usage(
            subpass_setup.device(),
            [TEXT_CACHE_WIDTH as u32, TEXT_CACHE_HEIGHT as u32],
            R8Unorm,
            ImageUsage {
                sampled: true,
                transfer_destination: true,
                .. ImageUsage::none()
            },
        ).unwrap();
        let text_cache_image_view = ImageView::new(text_cache_image.clone()).unwrap();

        let sampler = Sampler::new(
            subpass_setup.device(),
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Nearest,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            0.0, 1.0, 0.0, 0.0
        ).unwrap();

        let descriptor_set = Arc::new(
            PersistentDescriptorSet::start(pipeline.descriptor_set_layout(0).unwrap().clone())
                .add_sampled_image(text_cache_image_view, sampler).unwrap()
                .build().unwrap()
        );

        GuiPipeline { pipeline, descriptor_set, square_vertex_buffer, text_cache_image }
    }
}

impl Pipeline for GuiPipeline {
    fn raw_pipeline(&self) -> PipelineArc { self.pipeline.clone() }
}

const TEXT_CACHE_WIDTH: usize = 1000;
const TEXT_CACHE_HEIGHT: usize = 1000;

enum DrawVertexBuffer {
    Raw(Arc<ImmutableBuffer<[Vertex]>>),
    Text(Arc<TextDrawable>),
}

pub struct DrawCommand {
    vertex_buffer: DrawVertexBuffer,
    position: [f32; 2],
    size: [f32; 2],
    color: [f32; 4],
}

pub struct SizedDrawable {
    vertex_buffer: Arc<ImmutableBuffer<[Vertex]>>,
}

impl SizedDrawable {
    pub fn draw(&self, rect: Rect, color: Color) -> DrawCommand {
        DrawCommand {
            vertex_buffer: DrawVertexBuffer::Raw(self.vertex_buffer.clone()),
            position: rect.position.into(),
            size: rect.size.into(),
            color: encode_color(color),
        }
    }
}

pub struct TextDrawable {
    text_glyphs: Vec<PositionedGlyph<'static>>,
    width: f32,
    v_metrics: rusttype::VMetrics,
}

type TextBuffer = Arc<CpuAccessibleBuffer<[Vertex]>>;

impl TextDrawable {
    fn new(text_glyphs: Vec<PositionedGlyph<'static>>) -> TextDrawable {
        let last_glyph = text_glyphs.last().unwrap();
        let width = last_glyph.position().x + last_glyph.unpositioned().h_metrics().advance_width;
        let v_metrics = last_glyph.font().v_metrics(last_glyph.scale());
        TextDrawable { text_glyphs, width, v_metrics }
    }

    pub fn width(&self) -> f32 { self.width }
    pub fn ascent(&self) -> f32 { self.v_metrics.ascent }
    pub fn height(&self) -> f32 { self.v_metrics.ascent - self.v_metrics.descent }

    pub fn draw(self: Arc<TextDrawable>, position: Point, color: Color) -> DrawCommand {
        DrawCommand {
            vertex_buffer: DrawVertexBuffer::Text(self),
            position: position.into(),
            size: [1., 1.],
            color: encode_color(color),
        }
    }
}

pub struct DrawContext<'a> {
    builder: &'a mut AutoCommandBufferBuilder,
    subpass: &'a mut GuiSubpass,
    text_changed: bool,
}

impl<'a> DrawContext<'a> {
    pub fn color_rect_drawable(&mut self) -> Arc<SizedDrawable> {
        self.subpass.square_drawable.clone()
    }
    pub fn text_drawable(&mut self, font_name: Option<&str>, size: f32, text: &str) -> Arc<TextDrawable> {
        if text.is_empty() { panic!("text_drawable requires a non-empty string"); }
        let font = fonts().get(font_name);
        let glyphs: Vec<PositionedGlyph> = font.layout(text, Scale::uniform(size), point(0., 0.)).collect();
        let drawable = Arc::new(TextDrawable::new(glyphs));
        self.subpass.text_drawables.push((Arc::downgrade(&drawable), None));
        self.text_changed = true;
        drawable
    }

    fn update_cache(&mut self) {
        if self.text_changed {
            self.subpass.update_text_cache(self.builder);
            self.text_changed = false;
        }
    }
}

pub struct GuiSubpass {
    device: Arc<Device>,
    pipeline: GuiPipeline,
    screen_dimensions: Size,
    
    square_drawable: Arc<SizedDrawable>,
    text_drawables: Vec<(Weak<TextDrawable>, Option<TextBuffer>)>,
    text_cache: Cache<'static>,
    text_cache_pixel_buffer: Vec<u8>,
}

impl GuiSubpass {
    fn make_context<'a>(&'a mut self, builder: &'a mut AutoCommandBufferBuilder) -> DrawContext<'a> {
        DrawContext {
            builder, subpass: self, text_changed: false
        }
    }
    fn find_text_vertex_buffer(&self, drawable: Arc<TextDrawable>) -> TextBuffer {
        let drawable = Arc::downgrade(&drawable);
        for (weak, buffer) in self.text_drawables.iter() {
            if weak.ptr_eq(&drawable) {
                return buffer.as_ref().cloned().expect("TextDrawable has not been cached");
            }
        }
        panic!("TextDrawable not found in subpass");
    }

    fn update_text_cache(&mut self, builder: &mut AutoCommandBufferBuilder) {
        // Remove drawables that no longer exist.
        // TODO might want to do this somewhere else also to reclaim the buffers faster.
        self.text_drawables.retain(|text| text.0.strong_count() > 0);

        // Add visible drawables to the cache queue.
        for (drawable, _) in self.text_drawables.iter() {
            for glyph in drawable.upgrade().unwrap().text_glyphs.iter() {
                self.text_cache.queue_glyph(0, glyph.clone());
            }
        }
        // Update the cache, so that all glyphs in the queue are present in the cache texture.
        // TODO this builds the full cache image on the CPU, then reuploades the ENTIRE image whenever it changes.
        // should just allocate on the GPU and have cache_queued directly update the corresponding region.
        let cache = &mut self.text_cache;
        let cache_pixel_buffer = &mut self.text_cache_pixel_buffer;
        cache.cache_queued(
            |rect, src_data| {
                let width = (rect.max.x - rect.min.x) as usize;
                let height = (rect.max.y - rect.min.y) as usize;
                let mut dst_index = rect.min.y as usize * TEXT_CACHE_WIDTH + rect.min.x as usize;
                let mut src_index = 0;

                for _ in 0..height {
                    let dst_slice = &mut cache_pixel_buffer[dst_index..dst_index+width];
                    let src_slice = &src_data[src_index..src_index+width];
                    dst_slice.copy_from_slice(src_slice);

                    dst_index += TEXT_CACHE_WIDTH;
                    src_index += width;
                }
            }
        ).unwrap();

        let buffer = CpuAccessibleBuffer::<[u8]>::from_iter(
            self.device.clone(),
            BufferUsage::transfer_source(),
            false,
            cache_pixel_buffer.iter().cloned()
        ).unwrap();

        builder.copy_buffer_to_image(
            buffer.clone(),
            self.pipeline.text_cache_image.clone(),
        ).unwrap();

        // TODO might be able to reuse some of these vertex buffers if cache_queued returns CachedBy::Adding
        for (drawable, buffer) in self.text_drawables.iter_mut() {
            let drawable = drawable.upgrade().unwrap();
            let vertices: Vec<Vertex> = drawable.text_glyphs.iter().flat_map(|g| {
                if let Ok(Some((uv_rect, screen_rect))) = cache.rect_for(0, g) {
                    vec!(
                        Vertex::new([screen_rect.min.x, screen_rect.max.y], [uv_rect.min.x, uv_rect.max.y]),
                        Vertex::new([screen_rect.min.x, screen_rect.min.y], [uv_rect.min.x, uv_rect.min.y]),
                        Vertex::new([screen_rect.max.x, screen_rect.min.y], [uv_rect.max.x, uv_rect.min.y]),

                        Vertex::new([screen_rect.max.x, screen_rect.min.y], [uv_rect.max.x, uv_rect.min.y]),
                        Vertex::new([screen_rect.max.x, screen_rect.max.y], [uv_rect.max.x, uv_rect.max.y]),
                        Vertex::new([screen_rect.min.x, screen_rect.max.y], [uv_rect.min.x, uv_rect.max.y]),
                    ).into_iter()
                }
                else {
                    vec!().into_iter()
                }
            }).collect();

            // TODO use CpuBufferPool
            *buffer = Some(CpuAccessibleBuffer::from_iter(self.device.clone(), BufferUsage::vertex_buffer(), false, vertices.into_iter()).unwrap());
        }
    }
}

impl RenderSubpass for GuiSubpass {
    type SubpassCategory = subpass::Gui;
    type Scene = Gui;
    fn contents() -> SubpassContents { SubpassContents::Inline }
    fn new(subpass_setup: &mut SubpassSetup) -> Self {
        let text_cache = Cache::builder().dimensions(TEXT_CACHE_WIDTH as u32, TEXT_CACHE_HEIGHT as u32).build();
        let text_cache_pixel_buffer = vec!(0; TEXT_CACHE_WIDTH * TEXT_CACHE_HEIGHT);

        let pipeline = GuiPipeline::new(subpass_setup);
        let square_drawable = Arc::new(SizedDrawable { vertex_buffer: pipeline.square_vertex_buffer.clone() });
        GuiSubpass {
            device: subpass_setup.device(),
            pipeline,
            screen_dimensions: Size::zero(),

            square_drawable,
            text_drawables: Vec::new(),
            text_cache,
            text_cache_pixel_buffer,
        }
    }
    fn set_dimensions(&mut self, dimensions: Size) {
        self.screen_dimensions = dimensions;
    }

    fn pre_render(&mut self, scene: &mut Gui, builder: &mut AutoCommandBufferBuilder, _queue_family: QueueFamily) {
        let mut context = self.make_context(builder);
        scene.refresh_drawables(&mut context);
        context.update_cache();
        scene.refresh_layout(self.screen_dimensions);
    }

    fn render(&mut self, scene: &Gui, builder: &mut AutoCommandBufferBuilder, dynamic_state: &DynamicState) {
        let mut visitor = DrawCommandVisitor {
            subpass: self,
            builder,
            dynamic_state,
            screen_dimensions: self.screen_dimensions.into(),
        };
        visitor.walk(scene, scene.root_node(), Point::default());
    }
}

struct DrawCommandVisitor<'a> {
    subpass: &'a GuiSubpass,
    builder: &'a mut AutoCommandBufferBuilder,
    dynamic_state: &'a DynamicState,
    screen_dimensions: [f32; 2],
}

// TODO instead of one draw call per DrawCommand, build an instance buffer
impl<'a> DrawCommandVisitor<'a> {
    fn visit(&mut self, scene: &Gui, node: Node, parent_position: Point) -> Point {
        let (node_position, draw_command) = scene.draw_widget(parent_position, node);
        if let Some(draw_command) = draw_command {
            let vertex_buffer: Arc<dyn BufferAccess + Send + Sync> = match draw_command.vertex_buffer {
                DrawVertexBuffer::Raw(buffer) => buffer,
                DrawVertexBuffer::Text(drawable) => self.subpass.find_text_vertex_buffer(drawable),
            };
            let push_constants = vs::ty::PushConstants {
                screen_size: self.screen_dimensions,
                position: draw_command.position,
                size: draw_command.size,
                color: draw_command.color,
                _dummy0: [0; 8],
            };
            self.builder.draw(
                self.subpass.pipeline.raw_pipeline(),
                self.dynamic_state,
                vec![vertex_buffer],
                self.subpass.pipeline.descriptor_set.clone(),
                push_constants,
                vec![],
            ).unwrap();
        }
        node_position
    }
    fn walk(&mut self, gui: &Gui, node: Node, parent_position: Point) {
        let node_position = self.visit(gui, node, parent_position);
        for child in gui.children(node).unwrap() {
            self.walk(gui, child, node_position);
        }
    }
}
