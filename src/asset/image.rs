use std::io;
use std::fs::File;
use std::path::PathBuf;

use crate::geometry2d::*;
use super::{Asset, SimpleAsset, AssetCategory, AssetResult, AssetError};

// -------------------------------------------------------------------------------------------------

type BufReader = io::BufReader<File>;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ImageFormat {
    RGB24,
    RGBA32,
}

impl ImageFormat {
    fn from(color_type: png::ColorType, bit_depth: png::BitDepth) -> AssetResult<ImageFormat> {
        if bit_depth != png::BitDepth::Eight {
            return Err(AssetError::new_format(format!("invalid bit depth: {:?}", bit_depth)));
        }
        match color_type {
            png::ColorType::RGB => Ok(ImageFormat::RGB24),
            png::ColorType::RGBA => Ok(ImageFormat::RGBA32),
            _ => Err(AssetError::new_format(format!("invalid color type: {:?}", color_type))),
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

impl SimpleAsset for Image {
    fn category() -> AssetCategory { AssetCategory::Asset }
    fn file_extension() -> &'static str { "png" }
    fn load(reader: BufReader) -> AssetResult<Self> {
        let decoder = png::Decoder::new(reader);
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
    slice: EdgeRect,
}

impl Asset for NineSliceImage {
    fn category() -> AssetCategory { AssetCategory::Asset }
    fn file_extension() -> &'static str { "9slice.ron" }
    fn load(mut file_path: PathBuf) -> AssetResult<Self> {
        let reader = BufReader::new(File::open(&file_path)?);
        let slice = ron::de::from_reader(reader).map_err(|err| AssetError::new_format(err.to_string()))?;
        file_path.set_extension(<Image as Asset>::file_extension());
        let image = <Image as Asset>::load(file_path)?;
        Ok(NineSliceImage { image, slice })
    }
}
