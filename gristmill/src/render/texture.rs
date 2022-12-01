use crate::{asset::Asset, geom2d::Size, render::RenderContext};
use image::DynamicImage;
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, RwLock},
};
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
            dimensions.into(),
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

static TEXTURE_STORAGE_ASSETS: Lazy<TextureStorage> = Lazy::new(|| TextureStorage::new("assets"));

#[derive(Clone)]
pub struct TextureStorage {
    prefix: &'static str,
    textures: Arc<RwLock<HashMap<String, Texture>>>,
}

impl TextureStorage {
    pub fn new(prefix: &'static str) -> Self {
        TextureStorage {
            prefix,
            textures: Arc::default(),
        }
    }
    pub fn assets() -> &'static Self {
        &TEXTURE_STORAGE_ASSETS
    }

    pub fn load(&self, context: &mut RenderContext, asset_path: &str) -> Option<Texture> {
        let mut write_guard = self.textures.try_write().unwrap();
        if let Some(texture) = write_guard.get(asset_path) {
            Some(texture.clone())
        } else if let Some(image) = DynamicImage::load(self.prefix, asset_path) {
            let texture = Texture::load(context, &image);
            write_guard.insert(asset_path.to_owned(), texture.clone());
            Some(texture)
        } else {
            log::error!("Failed to load texture \"{}\".", asset_path);
            None
        }
    }
    pub fn get(&self, asset_path: &str) -> Option<Texture> {
        if let Some(texture) = self.textures.try_read().unwrap().get(asset_path) {
            Some(texture.clone())
        } else {
            log::error!("Texture \"{}\" hasn't been loaded.", asset_path);
            None
        }
    }
}
