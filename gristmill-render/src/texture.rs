use crate::RenderContext;
use gristmill_core::{
    asset::{self, image::DynamicImage, AssetError, AssetResult},
    math::IVec2,
};
use std::{hash::Hash, sync::Arc};
use vulkano::{
    format::Format,
    image::view::{ImageView, ImageViewCreateInfo},
    image::{ImageAccess, ImageDimensions, ImageViewAbstract, ImmutableImage, MipmapsCount},
    sampler::{ComponentMapping, ComponentSwizzle},
};

#[allow(clippy::derive_hash_xor_eq)]
#[derive(Clone, Hash)]
pub struct Texture(Arc<dyn ImageViewAbstract>);

impl Texture {
    pub fn load_image(context: &mut RenderContext, image: &DynamicImage) -> AssetResult<Self> {
        let allocator = context.allocator().clone();
        let (format, component_mapping) = Self::format_info(image);
        let vk_image = ImmutableImage::from_iter(
            &allocator,
            image.as_bytes().iter().cloned(),
            ImageDimensions::Dim2d {
                width: image.width(),
                height: image.height(),
                array_layers: 1,
            },
            MipmapsCount::One,
            format,
            context.builder(),
        )
        .map_err(|error| AssetError::Other(error.to_string()))?;
        let mut image_info = ImageViewCreateInfo::from_image(&vk_image);
        image_info.component_mapping = component_mapping;
        let image_view = ImageView::new(vk_image, image_info)
            .map_err(|error| AssetError::Other(error.to_string()))?;
        Ok(Texture(image_view))
    }
    pub fn load_asset(context: &mut RenderContext, file: &str) -> AssetResult<Self> {
        let image = asset::load_image_file("assets", file)?;
        Self::load_image(context, &image)
    }

    pub fn image(&self) -> Arc<dyn ImageAccess> {
        self.0.image()
    }
    pub fn image_view(&self) -> &Arc<dyn ImageViewAbstract> {
        &self.0
    }
    pub fn dimensions(&self) -> IVec2 {
        if let ImageDimensions::Dim2d { width, height, .. } = self.0.dimensions() {
            IVec2::new(width as i32, height as i32)
        } else {
            panic!("Texture is not 2D");
        }
    }

    fn format_info(image: &DynamicImage) -> (Format, ComponentMapping) {
        match *image {
            DynamicImage::ImageLuma8(_) => (
                Format::R8_SRGB,
                ComponentMapping {
                    r: ComponentSwizzle::Red,
                    g: ComponentSwizzle::Red,
                    b: ComponentSwizzle::Red,
                    a: ComponentSwizzle::One,
                },
            ),
            DynamicImage::ImageLumaA8(_) => (
                Format::R8G8_SRGB,
                ComponentMapping {
                    r: ComponentSwizzle::Red,
                    g: ComponentSwizzle::Red,
                    b: ComponentSwizzle::Red,
                    a: ComponentSwizzle::Green,
                },
            ),
            DynamicImage::ImageRgb8(_) => (Format::R8G8B8_SRGB, ComponentMapping::identity()),
            DynamicImage::ImageRgba8(_) => (Format::R8G8B8A8_SRGB, ComponentMapping::identity()),
            DynamicImage::ImageLuma16(_) => (
                Format::R16_UINT,
                ComponentMapping {
                    r: ComponentSwizzle::Red,
                    g: ComponentSwizzle::Red,
                    b: ComponentSwizzle::Red,
                    a: ComponentSwizzle::One,
                },
            ),
            DynamicImage::ImageLumaA16(_) => (
                Format::R16G16_UINT,
                ComponentMapping {
                    r: ComponentSwizzle::Red,
                    g: ComponentSwizzle::Red,
                    b: ComponentSwizzle::Red,
                    a: ComponentSwizzle::Green,
                },
            ),
            DynamicImage::ImageRgb16(_) => (Format::R16G16B16_UINT, ComponentMapping::identity()),
            DynamicImage::ImageRgba16(_) => {
                (Format::R16G16B16A16_UINT, ComponentMapping::identity())
            }
            DynamicImage::ImageRgb32F(_) => {
                (Format::R32G32B32_SFLOAT, ComponentMapping::identity())
            }
            DynamicImage::ImageRgba32F(_) => {
                (Format::R32G32B32A32_SFLOAT, ComponentMapping::identity())
            }
            _ => panic!("unknown image type"),
        }
    }
}

impl From<Arc<dyn ImageViewAbstract>> for Texture {
    fn from(image_view: Arc<dyn ImageViewAbstract>) -> Self {
        Texture(image_view)
    }
}

impl PartialEq for Texture {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.0, &other.0)
    }
}
impl Eq for Texture {}
