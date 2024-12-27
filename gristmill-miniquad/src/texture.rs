use std::{path::Path, sync::Mutex};

use gristmill::{
    asset::{Asset, AssetError, Image},
    Size,
};
use miniquad::{FilterMode, MipmapFilterMode, TextureId};

use crate::Context;

static DESTROY_TEXTURES: Mutex<Vec<TextureId>> = Mutex::new(Vec::new());

pub fn cleanup_textures(context: &mut Context) {
    let mut destroy = DESTROY_TEXTURES.lock().unwrap();
    for texture in destroy.drain(..) {
        context.delete_texture(texture);
    }
}

pub struct Texture(TextureId, Size);

impl Texture {
    pub fn load_image(context: &mut Context, image: &Image) -> Self {
        let texture = context.new_texture_from_rgba8(
            image.size.width as u16,
            image.size.height as u16,
            &image.data,
        );
        context.texture_set_filter(texture, FilterMode::Nearest, MipmapFilterMode::None);
        Texture(texture, image.size)
    }
    pub fn load(context: &mut Context, path: &Path) -> Result<Self, AssetError> {
        Ok(Self::load_image(context, &Image::load(path)?))
    }
    pub(crate) fn id(&self) -> TextureId {
        self.0
    }
    pub fn size(&self) -> Size {
        self.1
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        let mut destroy = DESTROY_TEXTURES.lock().unwrap();
        destroy.push(self.0);
    }
}
