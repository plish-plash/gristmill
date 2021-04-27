pub mod image;

use std::io;
use std::fs::File;
use std::path::PathBuf;

use serde::de::DeserializeOwned;

// -------------------------------------------------------------------------------------------------

pub enum AssetError {
    InvalidData,
    InvalidFormat(String),
    Inner(io::Error),
}

impl AssetError {
    pub fn new_data() -> AssetError { AssetError::InvalidData }
    pub fn new_format(message: String) -> AssetError { AssetError::InvalidFormat(message) }
    fn into_io_error(self) -> io::Error {
        // TODO add some context
        match self {
            AssetError::InvalidData => io::Error::from(io::ErrorKind::InvalidData),
            AssetError::InvalidFormat(message) => io::Error::new(io::ErrorKind::InvalidData, message),
            AssetError::Inner(err) => err,
        }
    }
}

impl From<io::Error> for AssetError {
    fn from(err: io::Error) -> AssetError {
        AssetError::Inner(err)
    }
}

pub type AssetResult<T> = Result<T, AssetError>;

// -------------------------------------------------------------------------------------------------

type BufReader = io::BufReader<File>;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum AssetCategory {
    Config, Asset, Resource
}

impl AssetCategory {
    fn file_prefix(&self) -> &'static str {
        match self {
            AssetCategory::Config => "config",
            AssetCategory::Asset => "assets",
            AssetCategory::Resource => "resources",
        }
    }
}

pub trait Asset: Sized {
    fn category() -> AssetCategory;
    fn file_extension() -> &'static str;
    fn load(file_path: PathBuf) -> AssetResult<Self>;
}

pub trait SimpleAsset: Sized {
    fn category() -> AssetCategory;
    fn file_extension() -> &'static str;
    fn load(reader: BufReader) -> AssetResult<Self>;
}

impl<T> Asset for T where T: SimpleAsset {
    fn category() -> AssetCategory { T::category() }
    fn file_extension() -> &'static str { T::file_extension() }
    fn load(file_path: PathBuf) -> AssetResult<Self> {
        let reader = BufReader::new(File::open(file_path)?);
        Self::load(reader)
    }
}

pub trait RonAsset: DeserializeOwned {
    fn category() -> AssetCategory;
}

impl<T> SimpleAsset for T where T: RonAsset {
    fn category() -> AssetCategory { T::category() }
    fn file_extension() -> &'static str { "ron" }
    fn load(reader: BufReader) -> AssetResult<Self> {
        ron::de::from_reader(reader).map_err(|err| AssetError::new_format(err.to_string()))
    }
}

// -------------------------------------------------------------------------------------------------

// Debug: expect working dir to be cargo project, so look for assets relative to that
#[cfg(debug_assertions)]
fn asset_base_path() -> io::Result<PathBuf> {
    Ok(PathBuf::new())
}

// Release: always look for assets relative to the executable
#[cfg(not(debug_assertions))]
fn asset_base_path() -> io::Result<PathBuf> {
    // TODO cache
    let mut dir = env::current_exe()?;
    dir.pop();
    Ok(dir)
}

pub fn load_asset<A: Asset>(asset_path: &str) -> io::Result<A> {
    let mut file_path = asset_base_path()?;
    file_path.push(A::category().file_prefix());
    file_path.push(asset_path);
    file_path.set_extension(A::file_extension());
    A::load(file_path).map_err(|err| err.into_io_error())
}
