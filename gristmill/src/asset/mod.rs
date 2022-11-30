pub mod image;

use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};
use std::{
    any::Any,
    collections::HashMap,
    fmt,
    fs::File,
    io::Error as IoError,
    path::{Path, PathBuf},
};

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

pub trait Asset: Send + Sync + Sized + 'static {
    fn read_from(reader: BufReader) -> AssetResult<Self>;
    fn load(prefix: &str, asset_path: &str) -> Option<Self> {
        let file_path = util::get_path(prefix, asset_path);
        match util::open_reader(&file_path).and_then(Self::read_from) {
            Ok(asset) => Some(asset),
            Err(error) => {
                log::warn!(
                    "Failed to load {}: {}",
                    file_path.to_str().unwrap_or(""),
                    error
                );
                None
            }
        }
    }
}

pub trait AssetWrite: Asset {
    fn write_to(value: &Self, writer: BufWriter) -> AssetResult<()>;
    fn save(value: &Self, prefix: &str, asset_path: &str) {
        let file_path = util::get_path(prefix, asset_path);
        match util::open_writer(&file_path).and_then(|writer| Self::write_to(value, writer)) {
            Ok(()) => (),
            Err(error) => log::error!(
                "Failed to save {}: {}",
                file_path.to_str().unwrap_or(""),
                error
            ),
        }
    }
}

pub mod util {
    use super::{asset_base_path, AssetError, AssetResult, BufReader, BufWriter};
    use serde::{de::DeserializeOwned, Serialize};
    use std::{
        fs::File,
        path::{Path, PathBuf},
    };

    pub(crate) fn get_path(prefix: &str, asset_path: &str) -> PathBuf {
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
    pub fn read_yaml<T: DeserializeOwned, R: std::io::Read>(reader: R) -> AssetResult<T> {
        serde_yaml::from_reader(reader).map_err(|err| AssetError::InvalidFormat(err.to_string()))
    }
    pub fn write_yaml<T: Serialize, W: std::io::Write>(writer: W, value: &T) -> AssetResult<()> {
        serde_yaml::to_writer(writer, value)
            .map_err(|err| AssetError::InvalidFormat(err.to_string()))
    }
}

static ASSET_STORAGE_CONFIG: Lazy<AssetStorage> = Lazy::new(|| AssetStorage::new("config"));
static ASSET_STORAGE_ASSETS: Lazy<AssetStorage> = Lazy::new(|| AssetStorage::new("assets"));

#[derive(Clone)]
pub struct AssetStorage {
    prefix: &'static str,
    assets: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
}

impl AssetStorage {
    pub fn new(prefix: &'static str) -> Self {
        AssetStorage {
            prefix,
            assets: Arc::default(),
        }
    }
    pub fn config() -> &'static Self {
        &ASSET_STORAGE_CONFIG
    }
    pub fn assets() -> &'static Self {
        &ASSET_STORAGE_ASSETS
    }

    fn try_load_asset<T: Asset>(&self, asset_path: &str, log_error: bool) {
        let mut write_guard = self.assets.write().unwrap();
        if !write_guard.contains_key(asset_path) {
            if let Some(asset) = T::load(self.prefix, asset_path) {
                write_guard.insert(asset_path.to_owned(), Box::new(asset));
            } else if log_error {
                log::error!("Failed to load asset \"{}\".", asset_path);
            }
        }
    }
    pub fn load<T>(&self, asset_path: &str) -> Option<T>
    where
        T: Asset + Clone,
    {
        self.try_load_asset::<T>(asset_path, true);
        self.assets
            .read()
            .unwrap()
            .get(asset_path)
            .and_then(|asset| {
                if let Some(a) = asset.downcast_ref::<T>() {
                    Some(a.clone())
                } else {
                    log::error!("Asset \"{}\" loaded as wrong type.", asset_path);
                    None
                }
            })
    }
    pub fn load_or_save_default<T, F>(&self, asset_path: &str, default: F) -> Option<T>
    where
        T: AssetWrite + Clone,
        F: FnOnce() -> T,
    {
        self.try_load_asset::<T>(asset_path, false);
        let mut write_guard = self.assets.write().unwrap();
        if !write_guard.contains_key(asset_path) {
            let new_asset = default();
            if !Path::exists(&util::get_path(self.prefix, asset_path)) {
                log::info!(
                    "Asset \"{}\" wasn't found, defaults will be saved",
                    asset_path
                );
                AssetWrite::save(&new_asset, self.prefix, asset_path);
            } else {
                log::warn!(
                    "Asset \"{}\" exists but wasn't loaded, defaults will be used instead",
                    asset_path
                );
            }
            write_guard.insert(asset_path.to_owned(), Box::new(new_asset));
        }
        drop(write_guard);
        self.load(asset_path)
    }
}
