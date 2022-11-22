use crate::asset::{category, Asset, AssetError, AssetExt, AssetResult};
use crate::geom2d::Size;

// -------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ImageFormat {
    Rgb24,
    Rgba32,
}

impl ImageFormat {
    fn from(color_type: png::ColorType, bit_depth: png::BitDepth) -> AssetResult<ImageFormat> {
        if bit_depth != png::BitDepth::Eight {
            return Err(AssetError::InvalidFormat(format!(
                "invalid bit depth: {:?}",
                bit_depth
            )));
        }
        match color_type {
            png::ColorType::Rgb => Ok(ImageFormat::Rgb24),
            png::ColorType::Rgba => Ok(ImageFormat::Rgba32),
            _ => Err(AssetError::InvalidFormat(format!(
                "invalid color type: {:?}",
                color_type
            ))),
        }
    }
}

impl From<ImageFormat> for vulkano::format::Format {
    fn from(format: ImageFormat) -> Self {
        match format {
            ImageFormat::Rgb24 => vulkano::format::Format::R8G8B8_SRGB,
            ImageFormat::Rgba32 => vulkano::format::Format::R8G8B8A8_SRGB,
        }
    }
}

pub struct Image {
    size: Size,
    format: ImageFormat,
    buffer: Vec<u8>,
}

impl Image {
    pub fn size(&self) -> Size {
        self.size
    }
    pub fn format(&self) -> ImageFormat {
        self.format
    }
    pub fn data(&self) -> &[u8] {
        &self.buffer
    }

    pub fn new_1x1_white() -> Image {
        Image {
            size: Size {
                width: 1,
                height: 1,
            },
            format: ImageFormat::Rgba32,
            buffer: vec![255, 255, 255, 255],
        }
    }
}

impl Asset for Image {
    type Category = category::Data;
    fn read(asset_path: &str) -> AssetResult<Self> {
        let decoder = png::Decoder::new(Self::open_file(asset_path, "png")?);
        let mut reader = decoder.read_info().unwrap();
        let mut buffer = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buffer).unwrap();
        buffer.truncate(info.buffer_size());
        let format = ImageFormat::from(info.color_type, info.bit_depth)?;
        Ok(Image {
            size: Size {
                width: info.width,
                height: info.height,
            },
            format,
            buffer,
        })
    }
}
