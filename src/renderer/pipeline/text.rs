use std::sync::Arc;

use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::descriptor::pipeline_layout::PipelineLayoutAbstract;
use vulkano::device::DeviceOwned;
use vulkano::format::R8Unorm;
use vulkano::image::{AttachmentImage, ImageUsage};
use vulkano::image::view::ImageView;
use vulkano::sampler::{Sampler, Filter, MipmapMode, SamplerAddressMode};

use rusttype::PositionedGlyph;
use rusttype::gpu_cache::Cache;

use crate::renderer::{PipelineArc, SubpassSetup};
use crate::util::handle::HandleOwner;
use crate::new_handle_type;

const TEXT_CACHE_WIDTH: u32 = 1000;
const TEXT_CACHE_HEIGHT: u32 = 1000;

pub trait Vertex {
    fn new(position: [i32; 2], tex_position: [f32; 2]) -> Self;
}

new_handle_type!(Handle);

struct Section<V: Vertex> {
    text_glyphs: Vec<PositionedGlyph<'static>>,
    vertex_buffer: Option<Arc<CpuAccessibleBuffer<[V]>>>,
}

pub struct PipelineData<V: Vertex> {
    cache: Cache<'static>,
    cache_pixel_buffer: Vec<u8>,
    cache_image: Arc<AttachmentImage<R8Unorm>>,
    descriptor_set: Arc<dyn DescriptorSet + Send + Sync>,
    sections: HandleOwner<Handle, Section<V>>,
}

impl<V> PipelineData<V> where V: Vertex + 'static {
    pub fn new(pipeline: PipelineArc, subpass_setup: &mut SubpassSetup) -> PipelineData<V> {
        let cache = Cache::builder().dimensions(TEXT_CACHE_WIDTH, TEXT_CACHE_HEIGHT).build();
        let cache_pixel_buffer = vec!(0; (TEXT_CACHE_WIDTH * TEXT_CACHE_HEIGHT) as usize);

        // Note: because this is an AttachmentImage, the color_attachment usage is added implicitly, which is unneeded.
        // However, Vulkano doesn't provide a "DeviceLocalImage", so this is what we've got.
        let cache_image = AttachmentImage::with_usage(
            subpass_setup.device(),
            [TEXT_CACHE_WIDTH, TEXT_CACHE_HEIGHT],
            R8Unorm,
            ImageUsage {
                sampled: true,
                transfer_destination: true,
                .. ImageUsage::none()
            },
        ).unwrap();
        let cache_image_view = ImageView::new(cache_image.clone()).unwrap();

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
                .add_sampled_image(cache_image_view, sampler).unwrap()
                .build().unwrap()
        );

        PipelineData { cache, cache_pixel_buffer, cache_image, descriptor_set, sections: HandleOwner::new() }
    }

    pub fn descriptor_set(&self) -> Arc<dyn DescriptorSet + Send + Sync> {
        self.descriptor_set.clone()
    }

    pub fn add_section(&mut self, text_glyphs: Vec<PositionedGlyph<'static>>) -> Arc<Handle> {
        self.sections.insert(Section { text_glyphs, vertex_buffer: None })
    }
    pub fn get_section_vertex_buffer(&self, handle: &Arc<Handle>) -> Arc<CpuAccessibleBuffer<[V]>> {
        self.sections.get(handle).vertex_buffer.as_ref().unwrap().clone()
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
            let vertices: Vec<V> = section.text_glyphs.iter().flat_map(|g| {
                if let Ok(Some((uv_rect, screen_rect))) = cache.rect_for(0, g) {
                    vec!(
                        V::new([screen_rect.min.x, screen_rect.max.y], [uv_rect.min.x, uv_rect.max.y]),
                        V::new([screen_rect.min.x, screen_rect.min.y], [uv_rect.min.x, uv_rect.min.y]),
                        V::new([screen_rect.max.x, screen_rect.min.y], [uv_rect.max.x, uv_rect.min.y]),

                        V::new([screen_rect.max.x, screen_rect.min.y], [uv_rect.max.x, uv_rect.min.y]),
                        V::new([screen_rect.max.x, screen_rect.max.y], [uv_rect.max.x, uv_rect.max.y]),
                        V::new([screen_rect.min.x, screen_rect.max.y], [uv_rect.min.x, uv_rect.max.y]),
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
