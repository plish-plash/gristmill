use crate::asset::{Asset, AssetError, AssetResult, BufReader};
use image::{io::Reader, ImageError};
pub use image::{DynamicImage, ImageBuffer};

impl Asset for DynamicImage {
    fn read_from(reader: BufReader) -> AssetResult<Self> {
        Ok(Reader::new(reader).with_guessed_format()?.decode()?)
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
