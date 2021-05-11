use serde::Deserialize;

use crate::geometry2d::*;
use super::{Asset, AssetExt, AssetResult, AssetError, category};

// -------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ImageFormat {
    RGB24,
    RGBA32,
}

impl ImageFormat {
    fn from(color_type: png::ColorType, bit_depth: png::BitDepth) -> AssetResult<ImageFormat> {
        if bit_depth != png::BitDepth::Eight {
            return Err(AssetError::InvalidFormat(format!("invalid bit depth: {:?}", bit_depth)));
        }
        match color_type {
            png::ColorType::RGB => Ok(ImageFormat::RGB24),
            png::ColorType::RGBA => Ok(ImageFormat::RGBA32),
            _ => Err(AssetError::InvalidFormat(format!("invalid color type: {:?}", color_type))),
        }
    }
}

impl From<ImageFormat> for vulkano::format::Format {
    fn from(format: ImageFormat) -> Self {
        match format {
            ImageFormat::RGB24 => vulkano::format::Format::R8G8B8Srgb,
            ImageFormat::RGBA32 => vulkano::format::Format::R8G8B8A8Srgb,
        }
    }
}

pub struct Image {
    size: Size,
    format: ImageFormat,
    buffer: Vec<u8>,
}

impl Image {
    pub fn size(&self) -> Size { self.size }
    pub fn format(&self) -> ImageFormat { self.format }
    pub fn data(&self) -> &[u8] { &self.buffer }

    pub fn new_1x1_white() -> Image {
        Image {
            size: Size { width: 1, height: 1 },
            format: ImageFormat::RGBA32,
            buffer: vec![255, 255, 255, 255],
        }
    }
}

impl Asset for Image {
    type Category = category::Data;
    fn read(asset_path: &str) -> AssetResult<Self> {
        let decoder = png::Decoder::new(Self::open_file(asset_path, "png")?);
        let (info, mut reader) = decoder.read_info().unwrap();
        let format = ImageFormat::from(info.color_type, info.bit_depth)?;
        let mut buffer = vec![0; info.buffer_size()];
        reader.next_frame(&mut buffer).unwrap();
        Ok(Image {
            size: Size { width: info.width, height: info.height },
            format,
            buffer,
        })
    }
}

pub struct NineSliceImage {
    image: Image,
    slices: EdgeRect,
}

impl NineSliceImage {
    pub fn size(&self) -> Size { self.image.size() }
    pub fn format(&self) -> ImageFormat { self.image.format() }
    pub fn data(&self) -> &[u8] { self.image.data() }
    pub fn slices(&self) -> EdgeRect { self.slices }
    pub fn as_image(&self) -> &Image { &self.image }
}

impl Asset for NineSliceImage {
    type Category = category::Data;
    fn read(asset_path: &str) -> AssetResult<Self> {
        let slices = Self::read_ron(asset_path)?;
        let image = Image::read(asset_path)?;
        Ok(NineSliceImage { image, slices })
    }
}

#[derive(Deserialize)]
struct TileAtlasInfo {
    tile_size: Size,
    tile_offset: Point,
    tile_gap: Point,
}

pub struct TileAtlasImage {
    image: Image,
    info: TileAtlasInfo,
}

impl TileAtlasImage {
    pub fn size(&self) -> Size { self.image.size() }
    pub fn format(&self) -> ImageFormat { self.image.format() }
    pub fn data(&self) -> &[u8] { self.image.data() }
    pub fn tile_size(&self) -> Size { self.info.tile_size }
    pub fn tile_offset(&self) -> Point { self.info.tile_offset }
    pub fn tile_gap(&self) -> Point { self.info.tile_gap }
    pub fn as_image(&self) -> &Image { &self.image }
}

impl Asset for TileAtlasImage {
    type Category = category::Data;
    fn read(asset_path: &str) -> AssetResult<Self> {
        let info = Self::read_ron(asset_path)?;
        let image = Image::read(asset_path)?;
        Ok(TileAtlasImage { image, info })
    }
}
