use crate::asset::{Asset, AssetCategory, AssetError, AssetResult, BufReader};
pub use image::*;

impl Asset for DynamicImage {
    fn category() -> AssetCategory {
        AssetCategory::ASSET
    }
    fn read_from(reader: BufReader) -> AssetResult<Self> {
        Ok(io::Reader::new(reader).with_guessed_format()?.decode()?)
    }
}
impl From<ImageError> for AssetError {
    fn from(error: ImageError) -> Self {
        if let ImageError::IoError(io_error) = error {
            AssetError::Io(io_error)
        } else {
            AssetError::InvalidFormat(format!("{}", error))
        }
    }
}
