use crate::RenderContext;
use gristmill_core::{
    asset::{image::DynamicImage, AssetExt, AssetStorage},
    geom2d::Size,
};
use std::{hash::Hash, sync::Arc};
use vulkano::{
    format::Format,
    image::view::{ImageView, ImageViewCreateInfo},
    image::{ImageAccess, ImageDimensions, ImageViewAbstract, ImmutableImage, MipmapsCount},
    sampler::{ComponentMapping, ComponentSwizzle},
};

pub trait ImageDimensionsExt {
    fn from_size(size: Size) -> Self;
}
impl ImageDimensionsExt for ImageDimensions {
    fn from_size(size: Size) -> Self {
        ImageDimensions::Dim2d {
            width: size.width,
            height: size.height,
            array_layers: 1,
        }
    }
}

#[allow(clippy::derive_hash_xor_eq)]
#[derive(Clone, Hash)]
pub struct Texture(Arc<dyn ImageViewAbstract>);

impl Texture {
    pub fn image(&self) -> Arc<dyn ImageAccess> {
        self.0.image()
    }
    pub fn image_view(&self) -> &Arc<dyn ImageViewAbstract> {
        &self.0
    }
    pub fn dimensions(&self) -> Size {
        if let ImageDimensions::Dim2d { width, height, .. } = self.0.dimensions() {
            Size { width, height }
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
    pub fn load(context: &mut RenderContext, image: &DynamicImage) -> Texture {
        let allocator = context.allocator().clone();
        let dimensions = Size::new(image.width(), image.height());
        let (format, component_mapping) = Self::format_info(image);
        let vk_image = ImmutableImage::from_iter(
            &allocator,
            image.as_bytes().iter().cloned(),
            ImageDimensions::from_size(dimensions),
            MipmapsCount::One,
            format,
            context.builder(),
        )
        .unwrap();
        let mut image_info = ImageViewCreateInfo::from_image(&vk_image);
        image_info.component_mapping = component_mapping;
        let image_view = ImageView::new(vk_image, image_info).unwrap();
        Texture(image_view)
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

pub trait TextureStorage {
    fn load(&mut self, context: &mut RenderContext, asset_path: &str) -> Option<&Texture>;
}

impl TextureStorage for AssetStorage<Texture> {
    fn load(&mut self, context: &mut RenderContext, asset_path: &str) -> Option<&Texture> {
        if !self.contains(asset_path) {
            if let Some(image) = DynamicImage::load(asset_path) {
                let asset = Texture::load(context, &image);
                self.insert(asset_path.to_owned(), asset);
            } else {
                log::error!("Failed to load texture \"{}\".", asset_path);
            }
        }
        self.get(asset_path)
    }
}
