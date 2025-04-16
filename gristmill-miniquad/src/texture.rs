use std::{
    any::TypeId,
    cell::RefCell,
    collections::HashMap,
    marker::PhantomData,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Mutex,
};

use gristmill::{
    asset::{sub_asset_path, Asset, AssetError, Image},
    color::Color,
    math::{Pos2, Rect, Vec2},
    scene2d::{Instance, UvRect},
    Size,
};
use miniquad::*;
use serde::Deserialize;

use crate::{Context, Material, Sprite2D};

static DESTROY_TEXTURES: Mutex<Vec<TextureId>> = Mutex::new(Vec::new());

pub fn cleanup_textures(context: &mut Context) {
    let mut destroy = DESTROY_TEXTURES.lock().unwrap();
    for texture in destroy.drain(..) {
        context.delete_texture(texture);
    }
}

pub struct TextureInner(TextureId, Size);

impl Drop for TextureInner {
    fn drop(&mut self) {
        let mut destroy = DESTROY_TEXTURES.lock().unwrap();
        destroy.push(self.0);
    }
}

#[derive(Deserialize, Clone)]
#[serde(try_from = "PathBuf")]
pub struct Texture(Rc<TextureInner>);

impl Texture {
    pub fn new(context: &mut Context, size: Size) -> Self {
        let texture = context.new_texture(
            TextureAccess::Static,
            TextureSource::Empty,
            TextureParams {
                kind: TextureKind::Texture2D,
                width: size.width as _,
                height: size.height as _,
                format: TextureFormat::RGBA8,
                wrap: TextureWrap::Clamp,
                min_filter: FilterMode::Nearest,
                mag_filter: FilterMode::Nearest,
                mipmap_filter: MipmapFilterMode::None,
                allocate_mipmaps: false,
                sample_count: 0,
            },
        );
        Texture(Rc::new(TextureInner(texture, size)))
    }
    pub fn from_image(context: &mut Context, image: &Image) -> Self {
        let texture = context.new_texture_from_rgba8(
            image.size.width as u16,
            image.size.height as u16,
            &image.data,
        );
        context.texture_set_filter(texture, FilterMode::Nearest, MipmapFilterMode::None);
        Texture(Rc::new(TextureInner(texture, image.size)))
    }
    pub fn write_part(&self, context: &mut Context, image: &Image, x: i32, y: i32) {
        context.texture_update_part(
            self.0 .0,
            x,
            y,
            image.size.width as i32,
            image.size.height as i32,
            &image.data,
        );
    }
    pub fn load(context: &mut Context, path: &Path) -> Result<Self, AssetError> {
        Ok(Self::from_image(context, &Image::load(path)?))
    }
    pub fn size(&self) -> Size {
        self.0 .1
    }
    pub fn material(&self) -> Material {
        Material(Some(self.0 .0))
    }
    pub fn sprite(&self, position: Pos2, color: Color) -> Sprite2D {
        let rect = Rect::from_center_size(position, self.size().to_vec2());
        Sprite2D {
            material: self.material(),
            instance: Instance {
                rect,
                uv: UvRect::default(),
                color,
            },
        }
    }
}
impl TryFrom<PathBuf> for Texture {
    type Error = AssetError;
    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        TextureLoader::load(sub_asset_path("textures", true, &value))
    }
}

pub trait AtlasInfo: 'static {
    const SIZE: u32;
    const TILE_SIZE: u32;
}

#[derive(Deserialize)]
#[serde(try_from = "PathBuf")]
pub struct AtlasTexture<I: AtlasInfo> {
    pub texture: Texture,
    pub frames: Vec<UvRect>,
    _marker: PhantomData<I>,
}
impl<I: AtlasInfo> AtlasTexture<I> {
    pub fn material(&self) -> Material {
        self.texture.material()
    }
    pub fn sprite(&self, frame: usize, position: Pos2, color: Color) -> Sprite2D {
        let rect = Rect::from_center_size(position, Vec2::splat(I::TILE_SIZE as f32));
        Sprite2D {
            material: self.material(),
            instance: Instance {
                rect,
                uv: self.frames[frame],
                color,
            },
        }
    }
}
impl<I: AtlasInfo> TryFrom<PathBuf> for AtlasTexture<I> {
    type Error = AssetError;
    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        TextureLoader::load_atlas(sub_asset_path("textures", true, &value))
    }
}
impl<I: AtlasInfo> Clone for AtlasTexture<I> {
    fn clone(&self) -> Self {
        Self {
            texture: self.texture.clone(),
            frames: self.frames.clone(),
            _marker: self._marker,
        }
    }
}

pub struct TextureLoader {
    context: Context,
    textures: HashMap<PathBuf, Texture>,
    atlases: HashMap<TypeId, (Texture, u32)>,
    atlas_textures: HashMap<TypeId, HashMap<PathBuf, Vec<UvRect>>>,
}

thread_local! {
    static TEXTURE_LOADER: RefCell<Option<TextureLoader>> = RefCell::new(None);
}

impl TextureLoader {
    pub fn begin(context: Context) {
        TEXTURE_LOADER.set(Some(TextureLoader {
            context,
            textures: HashMap::new(),
            atlases: HashMap::new(),
            atlas_textures: HashMap::new(),
        }));
    }
    pub fn end() -> Context {
        let loader = TEXTURE_LOADER.take().expect("not loading textures");
        loader.context
    }
    pub fn load(path: PathBuf) -> Result<Texture, AssetError> {
        TEXTURE_LOADER.with(|texture_loader| {
            let mut texture_loader = texture_loader.borrow_mut();
            let texture_loader = texture_loader.as_mut().expect("not loading textures");
            let texture = if let Some(texture) = texture_loader.textures.get(&path) {
                texture
            } else {
                let texture = Texture::load(&mut texture_loader.context, &path)?;
                texture_loader.textures.entry(path).or_insert(texture)
            };
            Ok(texture.clone())
        })
    }
    fn add_to_atlas<I: AtlasInfo>(
        context: &mut Context,
        atlases: &mut HashMap<TypeId, (Texture, u32)>,
        image: &Image,
    ) -> (Texture, UvRect) {
        assert!(
            image.size.width == I::TILE_SIZE && image.size.height == I::TILE_SIZE,
            "image wrong size for atlas"
        );
        let atlas = atlases.entry(TypeId::of::<I>()).or_insert_with(|| {
            log::debug!(
                "Creating {0}x{0} atlas texture: {1}",
                I::SIZE,
                std::any::type_name::<I>()
            );
            (Texture::new(context, Size::new(I::SIZE, I::SIZE)), 0)
        });
        let atlas_cols = I::SIZE / I::TILE_SIZE;
        let x = (atlas.1 % atlas_cols) * I::TILE_SIZE;
        let y = (atlas.1 / atlas_cols) * I::TILE_SIZE;
        assert!(x < I::SIZE && y < I::SIZE, "out of space in atlas");
        atlas.0.write_part(context, image, x as i32, y as i32);
        atlas.1 += 1;
        (
            atlas.0.clone(),
            UvRect::from_region(
                Rect::from_min_size(
                    Pos2::new(x as f32, y as f32),
                    Vec2::splat(I::TILE_SIZE as f32),
                ),
                Size::new(I::SIZE, I::SIZE),
            ),
        )
    }
    pub fn load_atlas<I: AtlasInfo>(path: PathBuf) -> Result<AtlasTexture<I>, AssetError> {
        TEXTURE_LOADER.with(|texture_loader| {
            let mut texture_loader = texture_loader.borrow_mut();
            let texture_loader = texture_loader.as_mut().expect("not loading textures");
            let atlas_textures = texture_loader
                .atlas_textures
                .entry(TypeId::of::<I>())
                .or_default();
            let texture = if let Some(frames) = atlas_textures.get(&path) {
                let texture = texture_loader
                    .atlases
                    .get(&TypeId::of::<I>())
                    .unwrap()
                    .0
                    .clone();
                AtlasTexture {
                    texture,
                    frames: frames.clone(),
                    _marker: PhantomData,
                }
            } else {
                let image = Image::load(&path)?;
                if image.size.width == I::TILE_SIZE && image.size.height == I::TILE_SIZE {
                    let (texture, frame) = Self::add_to_atlas::<I>(
                        &mut texture_loader.context,
                        &mut texture_loader.atlases,
                        &image,
                    );
                    let frames = vec![frame];
                    atlas_textures.insert(path, frames.clone());
                    AtlasTexture {
                        texture,
                        frames,
                        _marker: PhantomData,
                    }
                } else {
                    if image.size.width < I::TILE_SIZE || image.size.height < I::TILE_SIZE {
                        panic!("image too small for atlas");
                    }
                    let mut atlas_texture = None;
                    let mut frames = Vec::new();
                    let mut x = 0;
                    while x + I::TILE_SIZE <= image.size.width {
                        let subimage = image.subimage(x, 0, Size::new(I::TILE_SIZE, I::TILE_SIZE));
                        let (texture, frame) = Self::add_to_atlas::<I>(
                            &mut texture_loader.context,
                            &mut texture_loader.atlases,
                            &subimage,
                        );
                        atlas_texture = Some(texture);
                        frames.push(frame);
                        x += I::TILE_SIZE;
                    }
                    atlas_textures.insert(path, frames.clone());
                    AtlasTexture {
                        texture: atlas_texture.unwrap(),
                        frames,
                        _marker: PhantomData,
                    }
                }
            };
            Ok(texture)
        })
    }
}
