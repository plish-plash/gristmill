use std::sync::Arc;

use vulkano::buffer::{ImmutableBuffer, BufferUsage};
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::descriptor::pipeline_layout::PipelineLayoutAbstract;
use vulkano::format::Format;
use vulkano::image::{ImmutableImage, ImageDimensions, MipmapsCount};
use vulkano::image::view::ImageView;
use vulkano::sampler::{Sampler, Filter, MipmapMode, SamplerAddressMode};

use crate::asset::image::Image;
use crate::renderer::{PipelineArc, SubpassSetup};

pub trait Vertex {
    fn new(x: f32, y: f32) -> Self;
}

#[derive(Clone)]
pub struct Texture {
    descriptor_set: Arc<dyn DescriptorSet + Send + Sync>,
}

impl Texture {
    pub fn descriptor_set(&self) -> Arc<dyn DescriptorSet + Send + Sync> {
        self.descriptor_set.clone()
    }
}

pub struct PipelineData<V: Vertex> {
    pipeline: PipelineArc,
    square_vertex_buffer: Arc<ImmutableBuffer<[V]>>,
}

impl<V> PipelineData<V> where V: Vertex + Send + Sync + 'static {
    pub fn new(pipeline: PipelineArc, subpass_setup: &mut SubpassSetup) -> PipelineData<V> {
        let (square_vertex_buffer, setup_future) = ImmutableBuffer::from_iter(
            vec![
                V::new(0., 1.),
                V::new(0., 0.),
                V::new(1., 0.),
                V::new(1., 0.),
                V::new(1., 1.),
                V::new(0., 1.),
            ].into_iter(),
            BufferUsage::vertex_buffer(),
            subpass_setup.queue(),
        ).unwrap();
        subpass_setup.queue_join(setup_future);

        PipelineData { pipeline, square_vertex_buffer }
    }

    pub fn square_vertex_buffer(&self) -> Arc<ImmutableBuffer<[V]>> {
        self.square_vertex_buffer.clone()
    }
    
    pub fn upload_texture(&mut self, subpass_setup: &mut SubpassSetup, image: &Image, filter: Filter) -> Texture {
        let image_size = image.size();
        let dimensions = ImageDimensions::Dim2d {
            width: image_size.width,
            height: image_size.height,
            array_layers: 1,
        };
        let (image, setup_future): (Arc<ImmutableImage<Format>>, _) = ImmutableImage::from_iter(
            image.data().iter().cloned(),
            dimensions,
            MipmapsCount::One,
            image.format().into(),
            subpass_setup.queue(),
        ).unwrap();
        let image_view = ImageView::new(image.clone()).unwrap();
        subpass_setup.queue_join(setup_future);

        let sampler = Sampler::new(
            subpass_setup.device(),
            filter,
            filter,
            MipmapMode::Nearest,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            0.0, 1.0, 0.0, 0.0
        ).unwrap();

        let descriptor_set = Arc::new(
            PersistentDescriptorSet::start(self.pipeline.descriptor_set_layout(0).unwrap().clone())
                .add_sampled_image(image_view, sampler).unwrap()
                .build().unwrap()
        );
        Texture { descriptor_set }
    }
}
