pub mod image;

use std::{
    collections::HashMap,
    fmt,
    fs::File,
    io::Error as IoError,
    path::{Path, PathBuf},
    rc::Rc,
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

pub struct AssetCategory(&'static str);

impl AssetCategory {
    pub const CONFIG: AssetCategory = AssetCategory("config");
    pub const ASSET: AssetCategory = AssetCategory("assets");
    pub const SAVE: AssetCategory = AssetCategory("save");
    fn prefix(&self) -> &str {
        self.0
    }
}

pub trait Asset: Sized {
    fn category() -> AssetCategory;
    fn read_from(reader: BufReader) -> AssetResult<Self>;
}

pub trait AssetExt: Sized {
    fn load(asset_path: &str) -> Option<Self>;
}
impl<T: Asset> AssetExt for T {
    fn load(asset_path: &str) -> Option<Self> {
        let file_path = util::get_path(Self::category().prefix(), asset_path);
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
}

pub trait AssetWriteExt {
    fn save(value: &Self, asset_path: &str);
    fn load_or_save<F>(asset_path: &str, f: F) -> Self
    where
        F: FnOnce() -> Self;
}
impl<T: AssetWrite> AssetWriteExt for T {
    fn save(value: &Self, asset_path: &str) {
        let file_path = util::get_path(Self::category().prefix(), asset_path);
        match util::open_writer(&file_path).and_then(|writer| Self::write_to(value, writer)) {
            Ok(()) => (),
            Err(error) => log::error!(
                "Failed to save {}: {}",
                file_path.to_str().unwrap_or(""),
                error
            ),
        }
    }
    fn load_or_save<F>(asset_path: &str, f: F) -> Self
    where
        F: FnOnce() -> Self,
    {
        if let Some(asset) = <Self as AssetExt>::load(asset_path) {
            return asset;
        }
        let asset = f();
        if !Path::exists(&util::get_path(T::category().prefix(), asset_path)) {
            log::info!(
                "Asset \"{}\" wasn't found, defaults will be saved.",
                asset_path
            );
            Self::save(&asset, asset_path);
        } else {
            log::warn!(
                "Asset \"{}\" exists but wasn't loaded, defaults will be used instead.",
                asset_path
            );
        }
        asset
    }
}

pub struct AssetStorage<T>(HashMap<String, T>);

impl<T> Default for AssetStorage<T> {
    fn default() -> Self {
        AssetStorage(HashMap::new())
    }
}

impl<T> AssetStorage<T> {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn contains(&self, asset_path: &str) -> bool {
        self.0.contains_key(asset_path)
    }
    pub fn get(&self, asset_path: &str) -> Option<&T> {
        self.0.get(asset_path)
    }
    pub fn insert(&mut self, asset_path: String, asset: T) -> Option<T> {
        self.0.insert(asset_path, asset)
    }
}

impl<T: Asset> AssetStorage<T> {
    pub fn load(&mut self, asset_path: &str) -> Option<&T> {
        if !self.contains(asset_path) {
            if let Some(asset) = T::load(asset_path) {
                self.insert(asset_path.to_owned(), asset);
            } else {
                log::error!("Failed to load asset \"{}\".", asset_path);
            }
        }
        self.get(asset_path)
    }
}

impl<T: Asset> AssetStorage<Rc<T>> {
    pub fn load(&mut self, asset_path: &str) -> Option<Rc<T>> {
        if !self.contains(asset_path) {
            if let Some(asset) = T::load(asset_path) {
                self.insert(asset_path.to_owned(), Rc::new(asset));
            } else {
                log::error!("Failed to load asset \"{}\".", asset_path);
            }
        }
        self.get(asset_path).cloned()
    }
}
