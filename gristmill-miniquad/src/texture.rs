use std::sync::{Arc, Mutex};

use emath::Vec2;
use gristmill::{asset::Image, render2d::Texture};
use miniquad::{FilterMode, MipmapFilterMode, TextureId};

use crate::RenderingContext;

enum TextureData {
    Image(Image),
    Texture(TextureId),
}

pub struct TextureAsset(Mutex<TextureData>);

impl TextureAsset {
    fn new(image: Image) -> Arc<Self> {
        Arc::new(TextureAsset(Mutex::new(TextureData::Image(image))))
    }
    fn load(&self, context: &mut RenderingContext) {
        let mut data = self.0.lock().unwrap();
        if let TextureData::Image(image) = &*data {
            let texture = context.new_texture_from_rgba8(
                image.size.0 as u16,
                image.size.1 as u16,
                &image.data,
            );
            context.texture_set_filter(texture, FilterMode::Nearest, MipmapFilterMode::None);
            *data = TextureData::Texture(texture);
        }
    }
    pub fn get(&self) -> Option<TextureId> {
        let data = self.0.lock().unwrap();
        match *data {
            TextureData::Image(_) => None,
            TextureData::Texture(texture_id) => Some(texture_id),
        }
    }
}

impl Drop for TextureAsset {
    fn drop(&mut self) {
        if let Some(texture) = self.get() {
            let mut destroy = DESTROY_TEXTURES.lock().unwrap();
            destroy.push(texture);
        }
    }
}

static CREATE_TEXTURES: Mutex<Vec<Arc<TextureAsset>>> = Mutex::new(Vec::new());
static DESTROY_TEXTURES: Mutex<Vec<TextureId>> = Mutex::new(Vec::new());

#[no_mangle]
fn load_texture(image: Image) -> Texture {
    let size = Vec2::new(image.size.0 as f32, image.size.1 as f32);
    let handle = TextureAsset::new(image);
    let mut create = CREATE_TEXTURES.lock().unwrap();
    create.push(handle.clone());
    Texture { handle, size }
}

pub fn update_textures(context: &mut RenderingContext) {
    let mut destroy = DESTROY_TEXTURES.lock().unwrap();
    if !destroy.is_empty() {
        if destroy.len() == 1 {
            log::trace!("Deleting 1 texture");
        } else {
            log::trace!("Deleting {} textures", destroy.len());
        }
        for texture in destroy.drain(..) {
            context.delete_texture(texture);
        }
    }
    let mut create = CREATE_TEXTURES.lock().unwrap();
    if !create.is_empty() {
        if create.len() == 1 {
            log::trace!("Creating 1 texture");
        } else {
            log::trace!("Creating {} textures", create.len());
        }
        for asset in create.drain(..) {
            asset.load(context);
        }
    }
}
