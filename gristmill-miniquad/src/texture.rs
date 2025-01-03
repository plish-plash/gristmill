use std::{
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Mutex,
};

use gristmill::{
    asset::{asset_relative_path, Asset, AssetError, Image},
    Size,
};
use miniquad::{FilterMode, MipmapFilterMode, TextureId};
use serde::Deserialize;

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

pub struct TextureLoader(Context, HashMap<PathBuf, Texture>);

thread_local! {
    static TEXTURE_LOADER: RefCell<Option<TextureLoader>> = RefCell::new(None);
}

impl TextureLoader {
    pub fn begin(context: Context) {
        TEXTURE_LOADER.set(Some(TextureLoader(context, HashMap::new())));
    }
    pub fn end() -> (Context, Vec<Texture>) {
        let loader = TEXTURE_LOADER.take().expect("not loading textures");
        (loader.0, loader.1.into_values().collect())
    }
    pub fn load(path: PathBuf) -> Result<TextureAsset, AssetError> {
        TEXTURE_LOADER.with(|texture_loader| {
            let mut texture_loader = texture_loader.borrow_mut();
            let texture_loader = texture_loader.as_mut().expect("not loading textures");
            let texture = if let Some(texture) = texture_loader.1.get(&path) {
                texture
            } else {
                let texture = Texture::load(&mut texture_loader.0, &path)?;
                texture_loader.1.entry(path).or_insert(texture)
            };
            Ok(TextureAsset(texture.id(), texture.size()))
        })
    }
}

#[derive(Deserialize, Clone)]
#[serde(try_from = "PathBuf")]
pub struct TextureAsset(TextureId, Size);

impl TextureAsset {
    pub(crate) fn id(&self) -> TextureId {
        self.0
    }
    pub fn size(&self) -> Size {
        self.1
    }
}
impl TryFrom<PathBuf> for TextureAsset {
    type Error = AssetError;
    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        TextureLoader::load(asset_relative_path(&value))
    }
}
