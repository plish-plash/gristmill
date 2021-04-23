use std::io;
use std::fs::File;
use std::path::PathBuf;

use crate::geometry2d::*;
use super::{Asset, SimpleAsset, AssetCategory, AssetResult, AssetError};

// -------------------------------------------------------------------------------------------------

type BufReader = io::BufReader<File>;

enum ImageFormat {
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

struct Image {
    size: Size,
    buffer: Vec<u8>,
    format: ImageFormat,
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
            buffer,
            format,
        })
    }
}

struct NineSliceImage {
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
