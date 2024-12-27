use std::{
    fs::File,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::Size;

pub type BufReader = std::io::BufReader<File>;
pub type BufWriter = std::io::BufWriter<File>;

#[derive(Debug)]
enum ErrorKind {
    IoError(std::io::Error),
    Format,
    Other,
}

pub struct AssetError {
    path: PathBuf,
    write: bool,
    kind: ErrorKind,
    info: String,
}

impl AssetError {
    pub fn new_io(path: PathBuf, write: bool, error: std::io::Error) -> Self {
        AssetError {
            path,
            write,
            kind: ErrorKind::IoError(error),
            info: String::new(),
        }
    }
    pub fn new_format<E: ToString>(path: PathBuf, write: bool, error: E) -> Self {
        AssetError {
            path,
            write,
            kind: ErrorKind::Format,
            info: error.to_string(),
        }
    }
    fn new_png(path: PathBuf, error: png::DecodingError) -> Self {
        match error {
            png::DecodingError::IoError(error) => AssetError {
                path,
                write: false,
                kind: ErrorKind::IoError(error),
                info: String::new(),
            },
            png::DecodingError::Format(error) => AssetError {
                path,
                write: false,
                kind: ErrorKind::Format,
                info: error.to_string(),
            },
            _ => AssetError {
                path,
                write: false,
                kind: ErrorKind::Other,
                info: error.to_string(),
            },
        }
    }

    pub fn not_found(&self) -> bool {
        if let ErrorKind::IoError(error) = &self.kind {
            error.kind() == std::io::ErrorKind::NotFound
        } else {
            false
        }
    }
}

impl std::fmt::Debug for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let read_write = if self.write { "writing" } else { "reading" };
        write!(f, "Error {} {}: ", read_write, self.path.to_string_lossy())?;
        if let ErrorKind::IoError(error) = &self.kind {
            if self.write && error.kind() == std::io::ErrorKind::NotFound {
                // When a NotFound error occurs while writing, it means a parent directory doesn't exist.
                write!(f, "The parent directory does not exist.")?;
                if let Some(code) = error.raw_os_error() {
                    write!(f, " (os error {code})")?;
                }
                Ok(())
            } else {
                write!(f, "{}", error)
            }
        } else {
            write!(f, "{}", self.info)
        }
    }
}
impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::error::Error for AssetError {}

pub type Result<T> = std::result::Result<T, AssetError>;

pub trait Asset: Sized {
    fn load(path: &Path) -> Result<Self>;
}

pub fn load_file(path: &Path) -> Result<BufReader> {
    log::debug!("Load file: {}", path.display());
    let file = File::open(path).map_err(|e| AssetError::new_io(path.to_owned(), false, e))?;
    Ok(BufReader::new(file))
}

pub trait YamlAsset: serde::de::DeserializeOwned {}

impl<T: YamlAsset> Asset for T {
    fn load(path: &Path) -> Result<Self> {
        let reader = load_file(&path)?;
        serde_yml::from_reader(reader)
            .map_err(|e| AssetError::new_format(path.to_owned(), false, e))
    }
}

pub struct Image {
    pub size: Size,
    pub data: Vec<u8>,
}

impl Asset for Image {
    fn load(path: &Path) -> Result<Self> {
        use png::*;
        let reader = load_file(&path)?;
        let mut decoder = Decoder::new(reader);
        decoder.set_transformations(Transformations::ALPHA);
        let mut image_reader = decoder
            .read_info()
            .map_err(|e| AssetError::new_png(path.to_owned(), e))?;
        let mut data = vec![0; image_reader.output_buffer_size()];
        let info = image_reader
            .next_frame(&mut data)
            .map_err(|e| AssetError::new_png(path.to_owned(), e))?;
        data.truncate(info.buffer_size());
        assert_eq!(info.bit_depth, BitDepth::Eight);
        match info.color_type {
            ColorType::Rgba => {}
            ColorType::GrayscaleAlpha => {
                data = data
                    .chunks_exact(2)
                    .flat_map(|x| [x[0], x[0], x[0], x[1]])
                    .collect();
            }
            _ => {
                return Err(AssetError::new_format(
                    path.to_owned(),
                    false,
                    format!("unsupported color type {:?}", info.color_type),
                ))
            }
        }
        Ok(Image {
            size: Size::new(info.width, info.height),
            data,
        })
    }
}

#[derive(Deserialize)]
#[serde(transparent)]
pub struct AssetList(pub Vec<PathBuf>);
impl YamlAsset for AssetList {}

impl<T: Asset> Asset for Vec<T> {
    fn load(path: &Path) -> Result<Vec<T>> {
        let mut base_path = path.to_owned();
        base_path.pop();
        let list = AssetList::load(path)?;
        let mut assets = Vec::new();
        for asset_path in list.0 {
            let asset = T::load(&base_path.join(asset_path))?;
            assets.push(asset);
        }
        Ok(assets)
    }
}
