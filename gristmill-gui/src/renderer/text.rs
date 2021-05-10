use std::sync::Arc;

use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::descriptor::pipeline_layout::PipelineLayoutAbstract;
use vulkano::device::DeviceOwned;
use vulkano::format::R8Unorm;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::image::{StorageImage, ImageUsage, ImageDimensions, ImageCreateFlags};
use vulkano::image::view::ImageView;
use vulkano::sampler::{Sampler, Filter, MipmapMode, SamplerAddressMode};

use rusttype::PositionedGlyph;
use rusttype::gpu_cache::Cache;

use gristmill::renderer::{PipelineArc, LoadContext, RenderContext};
use gristmill::util::handle::HandleOwner;
use gristmill::new_handle_type;

const TEXT_CACHE_WIDTH: u32 = 1000;
const TEXT_CACHE_HEIGHT: u32 = 1000;

mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        src: "
            #version 450

            layout(push_constant) uniform PushConstants {
                vec2 screen_size;
                vec2 position;
                vec4 color;
            } constants;

            layout(location = 0) in vec2 position;
            layout(location = 1) in vec2 tex_position;
            layout(location = 0) out vec2 v_tex_position;
            layout(location = 1) out vec4 v_color;

            void main() {
                vec2 normalized_position = (constants.position + position) / constants.screen_size;
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
                f_color.a *= texture(tex, v_tex_position)[0];
            }
        "
    }
}

pub use vs::ty::PushConstants;

#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_position: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position, tex_position);

impl Vertex {
    fn new(position: [i32; 2], tex_position: [f32; 2]) -> Vertex {
        Vertex {
            position: [position[0] as f32, position[1] as f32],
            tex_position,
        }
    }
}

new_handle_type!(TextHandle);

struct TextSection {
    text_glyphs: Vec<PositionedGlyph<'static>>,
    vertex_buffer: Option<Arc<CpuAccessibleBuffer<[Vertex]>>>,
}

pub struct TextPipeline {
    pipeline: PipelineArc,
    cache: Cache<'static>,
    cache_pixel_buffer: Vec<u8>,
    cache_image: Arc<StorageImage<R8Unorm>>,
    descriptor_set: Arc<dyn DescriptorSet + Send + Sync>,
    sections: HandleOwner<TextHandle, TextSection>,
}

impl TextPipeline {
    pub fn new(context: &mut LoadContext) -> TextPipeline {
        let vs = vs::Shader::load(context.device()).unwrap();
        let fs = fs::Shader::load(context.device()).unwrap();

        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .blend_alpha_blending()
                .render_pass(context.subpass())
                .build(context.device())
                .unwrap()
        );

        let cache = Cache::builder().dimensions(TEXT_CACHE_WIDTH, TEXT_CACHE_HEIGHT).build();
        let cache_pixel_buffer = vec!(0; (TEXT_CACHE_WIDTH * TEXT_CACHE_HEIGHT) as usize);

        let cache_image = StorageImage::with_usage(
            context.device(),
            ImageDimensions::Dim2d { width: TEXT_CACHE_WIDTH, height: TEXT_CACHE_HEIGHT, array_layers: 1 },
            R8Unorm,
            ImageUsage {
                sampled: true,
                transfer_destination: true,
                .. ImageUsage::none()
            },
            ImageCreateFlags::none(),
            vec![context.queue().family()], // TODO should be BOTH graphics and transfer queue families.
        ).unwrap();
        let cache_image_view = ImageView::new(cache_image.clone()).unwrap();

        let sampler = Sampler::new(
            context.device(),
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
                .add_sampled_image(cache_image_view, sampler).unwrap()
                .build().unwrap()
        );

        TextPipeline { pipeline, cache, cache_pixel_buffer, cache_image, descriptor_set, sections: HandleOwner::new() }
    }

    pub fn draw(&self, context: &mut RenderContext, text: &Arc<TextHandle>, push_constants: PushConstants) {
        let vertex_buffer = self.sections.get(text).vertex_buffer.as_ref().expect("text cache needs update").clone();
        context.draw(
            self.pipeline.clone(),
            vec![vertex_buffer],
            self.descriptor_set.clone(),
            push_constants
        );
    }

    pub fn add_section(&mut self, text_glyphs: Vec<PositionedGlyph<'static>>) -> Arc<TextHandle> {
        self.sections.insert(TextSection { text_glyphs, vertex_buffer: None })
    }
    pub fn update_cache(&mut self, builder: &mut AutoCommandBufferBuilder) {
        self.sections.cleanup();

        for section in self.sections.iter() {
            for glyph in section.text_glyphs.iter() {
                self.cache.queue_glyph(0, glyph.clone());
            }
        }

        // Update the cache, so that all glyphs in the queue are present in the cache texture.
        // TODO this builds the full cache image on the CPU, then reuploades the ENTIRE image whenever it changes.
        // should just allocate on the GPU and have cache_queued directly update the corresponding region.
        let cache_pixel_buffer = &mut self.cache_pixel_buffer;
        self.cache.cache_queued(
            |rect, src_data| {
                let width = (rect.max.x - rect.min.x) as usize;
                let height = (rect.max.y - rect.min.y) as usize;
                let mut dst_index = ((rect.min.y * TEXT_CACHE_WIDTH) + rect.min.x) as usize;
                let mut src_index = 0;

                for _ in 0..height {
                    let dst_slice = &mut cache_pixel_buffer[dst_index..dst_index+width];
                    let src_slice = &src_data[src_index..src_index+width];
                    dst_slice.copy_from_slice(src_slice);

                    dst_index += TEXT_CACHE_WIDTH as usize;
                    src_index += width;
                }
            }
        ).unwrap();

        let buffer = CpuAccessibleBuffer::<[u8]>::from_iter(
            builder.device().clone(),
            BufferUsage::transfer_source(),
            false,
            cache_pixel_buffer.iter().cloned()
        ).unwrap();

        builder.copy_buffer_to_image(
            buffer.clone(),
            self.cache_image.clone(),
        ).unwrap();

        // TODO reuse valid vertex buffers if cache_queued returns CachedBy::Adding
        let cache = &self.cache;
        for section in self.sections.iter_mut() {
            let vertices: Vec<Vertex> = section.text_glyphs.iter().flat_map(|g| {
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
            section.vertex_buffer = Some(
                CpuAccessibleBuffer::from_iter(
                    builder.device().clone(),
                    BufferUsage::vertex_buffer(),
                    false,
                    vertices.into_iter(),
                ).unwrap());
        }
    }
}
