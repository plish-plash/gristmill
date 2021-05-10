use std::sync::Arc;

use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::descriptor::pipeline_layout::PipelineLayoutAbstract;
use vulkano::format::Format;
use vulkano::image::{ImmutableImage, ImageDimensions, MipmapsCount};
use vulkano::image::view::ImageView;
use vulkano::sampler::{Sampler, Filter, MipmapMode, SamplerAddressMode};

use crate::asset::image::Image;
use crate::geometry2d::*;
use crate::renderer::{PipelineArc, LoadContext};

#[derive(Clone)]
pub struct Texture {
    descriptor_set: Arc<dyn DescriptorSet + Send + Sync>,
    size: Size,
}

impl Texture {
    pub fn size(&self) -> Size { self.size }
    pub fn descriptor_set(&self) -> Arc<dyn DescriptorSet + Send + Sync> { self.descriptor_set.clone() }
}

pub struct TextureLoader;

impl TextureLoader {
    pub fn load_image(pipeline: &PipelineArc, context: &mut LoadContext, image: &Image, filter: Filter) -> Texture {
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
            context.queue(),
        ).unwrap();
        let image_view = ImageView::new(image.clone()).unwrap();
        context.load_future(setup_future);

        let sampler = Sampler::new(
            context.device(),
            filter,
            filter,
            MipmapMode::Nearest,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            0.0, 1.0, 0.0, 0.0
        ).unwrap();

        let descriptor_set = Arc::new(
            PersistentDescriptorSet::start(pipeline.descriptor_set_layout(0).unwrap().clone())
                .add_sampled_image(image_view, sampler).unwrap()
                .build().unwrap()
        );
        Texture { descriptor_set, size: image_size }
    }
}
