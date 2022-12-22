use crate::{RenderContext, Texture};
use bytemuck::{Pod, Zeroable};
use gristmill_core::{
    asset::image::{Rgba, RgbaImage},
    geom2d::Rect,
    Color,
};
use std::{cmp::Ordering, collections::HashMap, ptr::null, sync::Arc};
use vulkano::{
    buffer::{BufferUsage, CpuBufferPool, DeviceLocalBuffer},
    descriptor_set::{DescriptorSetWithOffsets, PersistentDescriptorSet, WriteDescriptorSet},
    image::ImageAccess,
    impl_vertex,
    memory::allocator::MemoryUsage,
    pipeline::{
        graphics::{
            color_blend::ColorBlendState,
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            vertex_input::BuffersDefinition,
            viewport::ViewportState,
        },
        GraphicsPipeline, Pipeline, PipelineBindPoint,
    },
    sampler::Sampler,
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
struct Instance {
    rect: [f32; 4],
    uv_rect: [f32; 4],
    color: [f32; 4],
}
impl_vertex!(Instance, rect, uv_rect, color);

#[derive(Clone)]
pub struct TextureRectPipeline {
    pipeline: Arc<GraphicsPipeline>,
    vertex_buffer: Arc<DeviceLocalBuffer<[Vertex; 4]>>,
    none_texture: Texture,
}

impl TextureRectPipeline {
    pub fn new(context: &mut RenderContext) -> Self {
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
        let allocator = context.allocator().clone();
        let vertex_buffer = DeviceLocalBuffer::from_data(
            &allocator,
            vertices,
            BufferUsage {
                vertex_buffer: true,
                ..BufferUsage::empty()
            },
            context.builder(),
        )
        .unwrap();

        let white_pixel: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
        let none_image = RgbaImage::from_pixel(1, 1, Rgba(white_pixel)).into();
        let none_texture = Texture::load(context, &none_image);

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

        TextureRectPipeline {
            pipeline,
            vertex_buffer,
            none_texture,
        }
    }
}

#[derive(Clone)]
pub struct TextureRect {
    pub texture: Option<Texture>,
    pub rect: Rect,
    pub uv_rect: Rect,
    pub color: Color,
    pub z: u16,
}

impl TextureRect {
    fn draw(&self, viewport: Rect) -> Instance {
        let viewport_extents = viewport.size / 2.0;
        Instance {
            rect: [
                (self.rect.position.x / viewport_extents.x) - 1.0,
                (self.rect.position.y / viewport_extents.y) - 1.0,
                self.rect.size.x / viewport_extents.x,
                self.rect.size.y / viewport_extents.y,
            ],
            uv_rect: self.uv_rect.into(),
            color: self.color.into(),
        }
    }
}

impl PartialEq for TextureRect {
    fn eq(&self, other: &Self) -> bool {
        self.texture == other.texture && self.z == other.z
    }
}
impl Eq for TextureRect {}

impl PartialOrd for TextureRect {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for TextureRect {
    fn cmp(&self, other: &Self) -> Ordering {
        match Ord::cmp(&self.z, &other.z) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => {
                let ptr = self
                    .texture
                    .as_ref()
                    .map(|t| Arc::as_ptr(t.image().inner().image))
                    .unwrap_or(null());
                let other_ptr = other
                    .texture
                    .as_ref()
                    .map(|t| Arc::as_ptr(t.image().inner().image))
                    .unwrap_or(null());
                Ord::cmp(&ptr, &other_ptr)
            }
        }
    }
}

pub struct TextureRectRenderer {
    pipeline: TextureRectPipeline,
    texture_descriptors: HashMap<Texture, DescriptorSetWithOffsets>,
    buffer_pool: CpuBufferPool<Instance>,
    instances: Vec<Instance>,
    draw_queue: Vec<TextureRect>,
}

impl TextureRectRenderer {
    pub fn new(context: &mut RenderContext) -> Self {
        TextureRectRenderer {
            pipeline: TextureRectPipeline::new(context),
            texture_descriptors: HashMap::new(),
            buffer_pool: CpuBufferPool::new(
                context.allocator().clone(),
                BufferUsage {
                    vertex_buffer: true,
                    ..BufferUsage::empty()
                },
                MemoryUsage::Upload,
            ),
            instances: Vec::new(),
            draw_queue: Vec::new(),
        }
    }

    pub fn remove(&mut self, texture: &Texture) {
        self.texture_descriptors.remove(texture);
    }

    pub fn queue(&mut self, rect: TextureRect) {
        self.draw_queue.push(rect);
    }
    pub fn queue_all<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = TextureRect>,
    {
        for rect in iter {
            self.draw_queue.push(rect);
        }
    }

    fn get_descriptor_set(
        &mut self,
        context: &mut RenderContext,
        texture: Texture,
    ) -> DescriptorSetWithOffsets {
        self.texture_descriptors
            .entry(texture.clone())
            .or_insert_with(|| {
                let layout = self
                    .pipeline
                    .pipeline
                    .layout()
                    .set_layouts()
                    .get(0)
                    .unwrap()
                    .clone();
                let sampler = Sampler::new(context.device(), Default::default()).unwrap();
                PersistentDescriptorSet::new(
                    context.descriptor_set_allocator(),
                    layout,
                    [WriteDescriptorSet::image_view_sampler(
                        0,
                        texture.image_view().clone(),
                        sampler,
                    )],
                )
                .unwrap()
                .into()
            })
            .clone()
    }
    fn draw_instances(&mut self, context: &mut RenderContext, texture: Option<Texture>) {
        const VERTEX_COUNT: u32 = 4;
        if self.instances.is_empty() {
            return;
        }
        let instance_count = self.instances.len() as u32;
        let instance_buffer = self
            .buffer_pool
            .from_iter(self.instances.drain(..))
            .unwrap();
        let descriptor_set = self.get_descriptor_set(
            context,
            texture.unwrap_or_else(|| self.pipeline.none_texture.clone()),
        );
        context
            .builder()
            .bind_vertex_buffers(0, (self.pipeline.vertex_buffer.clone(), instance_buffer))
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .draw(VERTEX_COUNT, instance_count, 0, 0)
            .unwrap();
    }
    pub fn draw_all(&mut self, context: &mut RenderContext) {
        context
            .builder()
            .bind_pipeline_graphics(self.pipeline.pipeline.clone());

        self.draw_queue.sort_unstable();
        let draw_queue = std::mem::take(&mut self.draw_queue);
        let viewport = context.viewport();
        let mut last_texture = None;
        for rect in draw_queue {
            if rect.texture != last_texture {
                self.draw_instances(context, last_texture);
                last_texture = rect.texture.clone();
            }
            self.instances.push(rect.draw(viewport));
        }
        self.draw_instances(context, last_texture);
    }
}
