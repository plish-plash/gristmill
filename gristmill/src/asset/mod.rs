pub mod image;

use std::{any::Any, collections::HashMap, fmt, io::Error as IoError, path::PathBuf};

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
    InvalidData,
    InvalidFormat(String),
    Io(IoError),
}

impl From<IoError> for AssetError {
    fn from(err: IoError) -> AssetError {
        AssetError::Io(err)
    }
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetError::InvalidData => write!(f, "invalid data"),
            AssetError::InvalidFormat(info) => write!(f, "{}", info),
            AssetError::Io(error) => write!(f, "{}", error),
        }
    }
}

pub type AssetResult<T> = Result<T, AssetError>;

pub trait Asset: Sized + 'static {
    fn extension() -> &'static str;
    fn read_from<R: std::io::Read>(reader: R) -> AssetResult<Self>;
}

pub trait AssetWrite: Asset {
    fn write_to<W: std::io::Write>(writer: W, value: &Self) -> AssetResult<()>;
}

pub mod util {
    use super::{asset_base_path, AssetError, AssetResult};
    use serde::{de::DeserializeOwned, Serialize};
    use std::fs::File;
    use std::path::{Path, PathBuf};

    pub type BufReader = std::io::BufReader<File>;
    pub type BufWriter = std::io::BufWriter<File>;

    pub(crate) fn get_path(prefix: &str, asset_path: &str, extension: &str) -> PathBuf {
        let mut file_path = asset_base_path();
        file_path.push(prefix);
        file_path.push(asset_path);
        file_path.set_extension(extension);
        file_path
    }
    pub fn open_reader(path: &Path) -> AssetResult<BufReader> {
        log::trace!("Reading file {}", path.to_string_lossy());
        Ok(BufReader::new(File::open(path)?))
    }
    pub fn open_writer(path: &Path) -> AssetResult<BufWriter> {
        log::trace!("Writing file {}", path.to_string_lossy());
        Ok(BufWriter::new(File::create(path)?))
    }
    pub fn read_ron<T: DeserializeOwned, R: std::io::Read>(reader: R) -> AssetResult<T> {
        ron::de::from_reader(reader).map_err(|err| AssetError::InvalidFormat(err.to_string()))
    }
    pub fn write_ron<T: Serialize, W: std::io::Write>(writer: W, value: &T) -> AssetResult<()> {
        use ron::ser::PrettyConfig;
        ron::ser::to_writer_pretty(writer, value, PrettyConfig::new())
            .map_err(|err| AssetError::InvalidFormat(err.to_string()))
    }
}

pub struct AssetStorage {
    prefix: &'static str,
    assets: HashMap<String, Box<dyn Any>>,
}

impl AssetStorage {
    pub fn new(prefix: &'static str) -> Self {
        AssetStorage {
            prefix,
            assets: HashMap::new(),
        }
    }
    pub fn config() -> Self {
        Self::new("config")
    }
    pub fn assets() -> Self {
        Self::new("assets")
    }
    pub fn get<T>(&mut self, asset_path: &str) -> Option<&T>
    where
        T: Asset,
    {
        if !self.assets.contains_key(asset_path) {
            let file_path = util::get_path(self.prefix, asset_path, T::extension());
            match util::open_reader(&file_path).and_then(T::read_from) {
                Ok(asset) => {
                    self.assets.insert(asset_path.to_owned(), Box::new(asset));
                }
                Err(error) => log::warn!("Failed to load {}: {}", asset_path, error),
            }
        }
        self.assets
            .get(asset_path)
            .map(|asset| asset.downcast_ref().expect("wrong type for asset"))
    }
    pub fn get_or_save<T, F>(&mut self, asset_path: &str, default: F) -> &T
    where
        T: AssetWrite,
        F: FnOnce() -> T,
    {
        self.get::<T>(asset_path);
        if !self.assets.contains_key(asset_path) {
            let file_path = util::get_path(self.prefix, asset_path, T::extension());
            log::info!(
                "{} not found, saving defaults to {}",
                asset_path,
                file_path.to_str().unwrap_or("")
            );
            let new_asset = default();
            match util::open_writer(&file_path).and_then(|writer| T::write_to(writer, &new_asset)) {
                Ok(()) => {}
                Err(error) => log::error!("Failed to save {}: {}", asset_path, error),
            }
            self.assets
                .insert(asset_path.to_owned(), Box::new(new_asset));
        }
        self.get(asset_path).unwrap()
    }
}
