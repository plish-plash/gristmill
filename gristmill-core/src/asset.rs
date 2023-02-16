use serde::{de::DeserializeOwned, Serialize};
use std::{
    fmt,
    fs::File,
    io::{Error as IoError, Read, Write},
    path::{Path, PathBuf},
};

pub use image;

pub type BufReader = std::io::BufReader<File>;
pub type BufWriter = std::io::BufWriter<File>;

// Debug: expect working dir to be cargo project, so look for assets relative to that
#[cfg(debug_assertions)]
fn asset_base_path() -> PathBuf {
    PathBuf::new()
}

// Release: always look for assets relative to the executable
#[cfg(not(debug_assertions))]
fn asset_base_path() -> PathBuf {
    // TODO cache this
    let mut dir = env::current_exe().unwrap();
    dir.pop();
    dir
}

#[derive(Debug)]
pub enum AssetError {
    Io(IoError),
    InvalidData,
    InvalidFormat(String),
    Other(String),
}

impl AssetError {
    pub fn io_kind(&self) -> Option<std::io::ErrorKind> {
        if let AssetError::Io(error) = self {
            Some(error.kind())
        } else {
            None
        }
    }
}

impl From<IoError> for AssetError {
    fn from(err: IoError) -> AssetError {
        AssetError::Io(err)
    }
}
impl From<image::ImageError> for AssetError {
    fn from(error: image::ImageError) -> Self {
        if let image::ImageError::IoError(io_error) = error {
            AssetError::Io(io_error)
        } else {
            AssetError::InvalidFormat(error.to_string())
        }
    }
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetError::Io(error) => write!(f, "{error}"),
            AssetError::InvalidData => write!(f, "invalid data"),
            AssetError::InvalidFormat(info) => write!(f, "{info}"),
            AssetError::Other(info) => write!(f, "{info}"),
        }
    }
}

pub type AssetResult<T> = Result<T, AssetError>;

pub fn get_path(prefix: &str, asset_path: &str) -> PathBuf {
    let mut file_path = asset_base_path();
    file_path.push(prefix);
    file_path.push(asset_path);
    file_path
}
pub fn open_reader(path: &Path) -> AssetResult<BufReader> {
    log::trace!("Reading file: {}", path.to_string_lossy());
    Ok(BufReader::new(File::open(path)?))
}
pub fn open_writer(path: &Path) -> AssetResult<BufWriter> {
    log::trace!("Writing file: {}", path.to_string_lossy());
    Ok(BufWriter::new(File::create(path)?))
}

pub fn load_text_file(prefix: &str, file: &str) -> Result<String, AssetError> {
    let path = get_path(prefix, file);
    let mut reader = open_reader(&path)?;
    let mut string = String::new();
    reader.read_to_string(&mut string)?;
    Ok(string)
}
pub fn save_text_file(prefix: &str, file: &str, value: &str) -> Result<(), AssetError> {
    let path = get_path(prefix, file);
    let mut writer = open_writer(&path)?;
    writer.write_all(value.as_bytes())?;
    Ok(())
}

pub fn load_yaml_file<T>(prefix: &str, file: &str) -> Result<T, AssetError>
where
    T: DeserializeOwned,
{
    let path = get_path(prefix, file);
    let reader = open_reader(&path)?;
    serde_yaml::from_reader(reader).map_err(|err| AssetError::InvalidFormat(err.to_string()))
}
pub fn save_yaml_file<T>(prefix: &str, file: &str, value: &T) -> Result<(), AssetError>
where
    T: Serialize,
{
    let path = get_path(prefix, file);
    let writer = open_writer(&path)?;
    serde_yaml::to_writer(writer, value).map_err(|err| AssetError::InvalidFormat(err.to_string()))
}

pub fn load_image_file(prefix: &str, file: &str) -> Result<image::DynamicImage, AssetError> {
    let path = get_path(prefix, file);
    log::trace!("Reading file: {}", path.to_string_lossy());
    Ok(image::io::Reader::open(&path)?.decode()?)
}
