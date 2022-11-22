use bytemuck::{Pod, Zeroable};
use glyph_brush::GlyphVertex;
use gristmill::{
    color::Pixel,
    geom2d::{Rect, Size},
    render::RenderContext,
};
use std::sync::Arc;
use vulkano::{
    buffer::{
        cpu_pool::CpuBufferPoolChunk, BufferUsage, CpuAccessibleBuffer, CpuBufferPool,
        DeviceLocalBuffer,
    },
    command_buffer::CopyBufferToImageInfo,
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    format::Format,
    image::{
        view::{ImageView, ImageViewCreateInfo},
        ImageAccess, ImageCreateFlags, ImageUsage, ImageViewAbstract, ImmutableImage, MipmapsCount,
        StorageImage,
    },
    impl_vertex,
    pipeline::{
        graphics::color_blend::ColorBlendState,
        graphics::input_assembly::{InputAssemblyState, PrimitiveTopology},
        graphics::vertex_input::BuffersDefinition,
        graphics::viewport::ViewportState,
        GraphicsPipeline, Pipeline, PipelineBindPoint,
    },
    sampler::{
        ComponentMapping, ComponentSwizzle, Filter, Sampler, SamplerAddressMode, SamplerCreateInfo,
    },
};

use crate::{
    render::{GuiDrawRect, GuiRenderBrush, Renderer},
    Gui, GuiTexture,
};

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450
            // vertex
            layout(location = 0) in vec2 position;
            // instance
            layout(location = 1) in vec4 rect;
            layout(location = 2) in vec4 uv_rect;
            layout(location = 3) in vec4 color;

            layout(location = 0) out vec2 v_uv;
            layout(location = 1) out vec4 v_color;

            void main() {
                gl_Position = vec4(rect.xy + (position * rect.zw), 0, 1);
                v_uv = uv_rect.xy + (abs(position) * uv_rect.zw);
                v_color = color;
            }"
    }
}
mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 450
            layout(location = 0) in vec2 v_uv;
            layout(location = 1) in vec4 v_color;

            layout(location = 0) out vec4 f_color;

            layout(set = 0, binding = 0) uniform sampler2D tex;

            void main() {
                f_color = texture(tex, v_uv) * v_color;
            }"
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
struct Vertex {
    position: [f32; 2],
}
impl_vertex!(Vertex, position);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Instance {
    rect: [f32; 4],
    uv_rect: [f32; 4],
    color: [f32; 4],
}
impl_vertex!(Instance, rect, uv_rect, color);

struct InstanceBuffer {
    instance_count: u32,
    buffer: Option<Arc<CpuBufferPoolChunk<Instance>>>,
    descriptor_set: Arc<PersistentDescriptorSet>,
}

impl InstanceBuffer {
    fn make_descriptor_set(
        context: &mut RenderContext,
        pipeline: &Arc<GraphicsPipeline>,
        image_view: Arc<dyn ImageViewAbstract>,
    ) -> Arc<PersistentDescriptorSet> {
        let sampler = Sampler::new(
            context.device(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                address_mode: [SamplerAddressMode::Repeat; 3],
                ..Default::default()
            },
        )
        .unwrap();
        let layout = pipeline.layout().set_layouts().get(0).unwrap();
        PersistentDescriptorSet::new(
            context.descriptor_set_allocator(),
            layout.clone(),
            [WriteDescriptorSet::image_view_sampler(
                0, image_view, sampler,
            )],
        )
        .unwrap()
    }
    fn new(
        context: &mut RenderContext,
        pipeline: &Arc<GraphicsPipeline>,
        image_view: Arc<dyn ImageViewAbstract>,
    ) -> Self {
        InstanceBuffer {
            instance_count: 0,
            buffer: None,
            descriptor_set: Self::make_descriptor_set(context, pipeline, image_view),
        }
    }
    fn replace_texture(
        &mut self,
        context: &mut RenderContext,
        pipeline: &Arc<GraphicsPipeline>,
        image_view: Arc<dyn ImageViewAbstract>,
    ) {
        self.descriptor_set = Self::make_descriptor_set(context, pipeline, image_view);
    }
}

pub struct GuiPipeline {
    pipeline: Arc<GraphicsPipeline>,
    vertex_buffer: Arc<DeviceLocalBuffer<[Vertex; 4]>>,
    buffer_pool: CpuBufferPool<Instance>,
    instance_buffers: Vec<InstanceBuffer>,
    glyph_texture: Option<(Arc<StorageImage>, GuiTexture)>,
}

impl GuiPipeline {
    fn new(context: &mut RenderContext) -> GuiPipeline {
        const VERTEX_BUFFER_USAGE: BufferUsage = BufferUsage {
            vertex_buffer: true,
            ..BufferUsage::empty()
        };

        let allocator = context.allocator();
        let vertices = [
            Vertex {
                position: [0.0, 0.0],
            },
            Vertex {
                position: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, 0.0],
            },
            Vertex {
                position: [1.0, 1.0],
            },
        ];
        let vertex_buffer = DeviceLocalBuffer::from_data(
            &allocator,
            vertices,
            VERTEX_BUFFER_USAGE,
            context.builder(),
        )
        .unwrap();

        let white_pixel: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
        let none_texture = ImmutableImage::from_iter(
            &allocator,
            white_pixel.into_iter(),
            Size::new(1, 1).into(),
            MipmapsCount::One,
            Format::R8G8B8A8_SRGB,
            context.builder(),
        )
        .unwrap();
        let none_image = ImageView::new_default(none_texture).unwrap();

        let vs = vs::load(context.device()).unwrap();
        let fs = fs::load(context.device()).unwrap();

        let subpass = context.render_pass();
        let pipeline = GraphicsPipeline::start()
            .vertex_input_state(
                BuffersDefinition::new()
                    .vertex::<Vertex>()
                    .instance::<Instance>(),
            )
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .input_assembly_state(
                InputAssemblyState::new().topology(PrimitiveTopology::TriangleStrip),
            )
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .color_blend_state(ColorBlendState::new(subpass.num_color_attachments()).blend_alpha())
            .render_pass(subpass)
            .build(context.device())
            .unwrap();

        let instance_buffers = vec![InstanceBuffer::new(context, &pipeline, none_image)];
        GuiPipeline {
            pipeline,
            vertex_buffer,
            buffer_pool: CpuBufferPool::vertex_buffer(allocator),
            instance_buffers,
            glyph_texture: None,
        }
    }
    fn add_texture(
        &mut self,
        context: &mut RenderContext,
        image_view: Arc<dyn ImageViewAbstract>,
    ) -> GuiTexture {
        let index = self.instance_buffers.len();
        self.instance_buffers
            .push(InstanceBuffer::new(context, &self.pipeline, image_view));
        GuiTexture(index)
    }
    fn render(&self, context: &mut RenderContext) {
        context
            .builder()
            .bind_pipeline_graphics(self.pipeline.clone());
        for instance_buffer in self.instance_buffers.iter() {
            if instance_buffer.instance_count > 0 {
                context
                    .builder()
                    .bind_vertex_buffers(
                        0,
                        (
                            self.vertex_buffer.clone(),
                            instance_buffer.buffer.as_ref().unwrap().clone(),
                        ),
                    )
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        self.pipeline.layout().clone(),
                        0,
                        instance_buffer.descriptor_set.clone(),
                    )
                    .draw(4, instance_buffer.instance_count, 0, 0)
                    .unwrap();
            }
        }
    }
}

impl Renderer for GuiPipeline {
    type Vertex = Instance;
    type Context = RenderContext;
    fn make_rect_vertex(&self, rect: GuiDrawRect) -> Self::Vertex {
        Instance {
            rect: [
                rect.rect.position.x as f32,
                rect.rect.position.y as f32,
                rect.rect.size.width as f32,
                rect.rect.size.height as f32,
            ],
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            color: rect.color.into_raw(),
        }
    }
    fn make_glyph_vertex(&self, glyph: GlyphVertex) -> Self::Vertex {
        fn to_f32_array(rect: glyph_brush::ab_glyph::Rect) -> [f32; 4] {
            [rect.min.x, rect.min.y, rect.width(), rect.height()]
        }
        Instance {
            rect: to_f32_array(glyph.pixel_coords),
            uv_rect: to_f32_array(glyph.tex_coords),
            color: glyph.extra.color,
        }
    }
    fn resize_glyph_texture(&mut self, context: &mut RenderContext, size: Size) -> GuiTexture {
        let usage = ImageUsage {
            transfer_dst: true,
            sampled: true,
            ..ImageUsage::empty()
        };
        let texture = StorageImage::with_usage(
            &context.allocator(),
            size.into(),
            Format::R8_SRGB,
            usage,
            ImageCreateFlags::empty(),
            [context.queue().queue_family_index()],
        )
        .unwrap();
        let mut image_info = ImageViewCreateInfo::from_image(&texture);
        image_info.component_mapping = ComponentMapping {
            r: ComponentSwizzle::One,
            g: ComponentSwizzle::One,
            b: ComponentSwizzle::One,
            a: ComponentSwizzle::Red,
        };
        let image_view = ImageView::new(texture.clone(), image_info).unwrap();
        if let Some(glyph_texture) = self.glyph_texture.as_mut() {
            glyph_texture.0 = texture;
            self.instance_buffers[glyph_texture.1 .0].replace_texture(
                context,
                &self.pipeline,
                image_view,
            );
            glyph_texture.1
        } else {
            let glyph_texture = self.add_texture(context, image_view);
            self.glyph_texture = Some((texture, glyph_texture));
            glyph_texture
        }
    }
    fn update_glyph_texture(&self, context: &mut RenderContext, region: Rect, tex_data: &[u8]) {
        const TRANSFER_BUFFER_USAGE: BufferUsage = BufferUsage {
            transfer_src: true,
            ..BufferUsage::empty()
        };
        let (glyph_texture, _) = self
            .glyph_texture
            .as_ref()
            .expect("glyph texture not created");
        let transfer_buffer = CpuAccessibleBuffer::from_iter(
            &context.allocator(),
            TRANSFER_BUFFER_USAGE,
            false,
            tex_data.iter().cloned(),
        )
        .unwrap();
        let mut copy_info =
            CopyBufferToImageInfo::buffer_image(transfer_buffer, glyph_texture.clone());
        copy_info.regions[0].image_offset = [region.position.x as u32, region.position.y as u32, 0];
        copy_info.regions[0].image_extent = [region.size.width, region.size.height, 1];
        context.builder().copy_buffer_to_image(copy_info).unwrap();
    }
    fn set_vertices(
        &mut self,
        _context: &mut RenderContext,
        texture: GuiTexture,
        mut vertices: Vec<Self::Vertex>,
        screen_size: Size,
    ) {
        assert_ne!(screen_size, Size::ZERO);
        let (half_width, half_height) = (
            screen_size.width as f32 / 2.0,
            screen_size.height as f32 / 2.0,
        );
        for vertex in vertices.iter_mut() {
            // Convert pixel coordinates to screen space.
            vertex.rect[0] = (vertex.rect[0] / half_width) - 1.0;
            vertex.rect[1] = (vertex.rect[1] / half_height) - 1.0;
            vertex.rect[2] /= half_width;
            vertex.rect[3] /= half_height;
        }
        self.instance_buffers[texture.0].instance_count = vertices.len() as u32;
        self.instance_buffers[texture.0].buffer =
            Some(self.buffer_pool.from_iter(vertices.into_iter()).unwrap());
    }
}

pub struct GuiRenderer {
    pipeline: GuiPipeline,
    render_brush: GuiRenderBrush<GuiPipeline>,
}

impl GuiRenderer {
    pub fn new(context: &mut RenderContext) -> GuiRenderer {
        let mut pipeline = GuiPipeline::new(context);
        let render_brush = GuiRenderBrush::new(context, &mut pipeline);
        GuiRenderer {
            pipeline,
            render_brush,
        }
    }
    pub fn add_texture(
        &mut self,
        context: &mut RenderContext,
        texture: Arc<dyn ImageAccess>,
    ) -> GuiTexture {
        let image_view = ImageView::new_default(texture).unwrap();
        self.pipeline.add_texture(context, image_view)
    }

    pub fn pre_render(&mut self, context: &mut RenderContext, gui: &mut Gui) {
        self.render_brush.set_viewport_rect(gui, context.viewport());
        self.render_brush.process(context, &mut self.pipeline, gui);
    }
    pub fn render(&mut self, context: &mut RenderContext) {
        self.pipeline.render(context);
    }
}
