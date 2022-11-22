pub mod image;
pub mod resource;

use serde::de::DeserializeOwned;
use std::{any::Any, collections::HashMap, fmt, fs::File, io, path::PathBuf};

// -------------------------------------------------------------------------------------------------

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

// -------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub enum AssetError {
    InvalidData,
    InvalidFormat(String),
    Io(io::Error),
}

impl From<io::Error> for AssetError {
    fn from(err: io::Error) -> AssetError {
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

// -------------------------------------------------------------------------------------------------

type BufReader = io::BufReader<File>;

pub trait AssetCategory {
    fn file_prefix() -> &'static str;
    fn get_file(asset_path: &str, extension: &str) -> PathBuf {
        let mut file_path = asset_base_path();
        file_path.push(Self::file_prefix());
        file_path.push(asset_path);
        file_path.set_extension(extension);
        file_path
    }
}

pub mod category {
    use super::AssetCategory;
    pub struct Config;
    impl AssetCategory for Config {
        fn file_prefix() -> &'static str {
            "config"
        }
    }
    pub struct Data;
    impl AssetCategory for Data {
        fn file_prefix() -> &'static str {
            "assets"
        }
    }
    pub struct Resource;
    impl AssetCategory for Resource {
        fn file_prefix() -> &'static str {
            "resources"
        }
    }
}

pub trait Asset: Sized {
    type Category: AssetCategory;
    fn read(asset_path: &str) -> AssetResult<Self>;
    fn try_read(asset_path: &str) -> Option<Self> {
        match Self::read(asset_path) {
            Ok(asset) => Some(asset),
            Err(error) => {
                log::error!("Failed to load asset {}: {}", asset_path, error);
                None
            }
        }
    }
}

pub trait AssetExt {
    fn get_file(asset_path: &str, extension: &str) -> PathBuf;
    fn open_file(asset_path: &str, extension: &str) -> AssetResult<BufReader>;
    fn read_ron<T: DeserializeOwned>(asset_path: &str) -> AssetResult<T>;
}

impl<T> AssetExt for T
where
    T: Asset,
{
    fn get_file(asset_path: &str, extension: &str) -> PathBuf {
        T::Category::get_file(asset_path, extension)
    }
    fn open_file(asset_path: &str, extension: &str) -> AssetResult<BufReader> {
        let file_path = Self::get_file(asset_path, extension);
        log::trace!("Opening file {}", file_path.to_string_lossy());
        Ok(BufReader::new(File::open(file_path)?))
    }
    fn read_ron<T1: DeserializeOwned>(asset_path: &str) -> AssetResult<T1> {
        let reader = Self::open_file(asset_path, "ron")?;
        ron::de::from_reader(reader).map_err(|err| AssetError::InvalidFormat(err.to_string()))
    }
}

#[macro_export]
macro_rules! impl_ron_asset {
    ($name:ident, $category:ident) => {
        impl $crate::asset::Asset for $name {
            type Category = $crate::asset::category::$category;
            fn read(asset_path: &str) -> $crate::asset::AssetResult<Self> {
                use $crate::asset::AssetExt;
                Self::read_ron(asset_path)
            }
        }
    };
}

// -------------------------------------------------------------------------------------------------

#[derive(Default)]
pub struct Resources {
    resources: HashMap<String, Box<dyn Any>>,
}

impl Resources {
    pub fn new() -> Resources {
        Self::default()
    }
    pub fn get<T>(&mut self, asset_path: &str) -> &T
    where
        T: Asset<Category = category::Resource> + Default + 'static,
    {
        if !self.resources.contains_key(asset_path) {
            let asset = match T::read(asset_path) {
                Ok(value) => {
                    log::debug!("Loaded resource {}", asset_path);
                    value
                }
                Err(error) => {
                    log::error!("Failed to load resource {}: {}", asset_path, error);
                    Default::default()
                }
            };
            self.resources
                .insert(asset_path.to_owned(), Box::new(asset));
        }

        return self
            .resources
            .get(asset_path)
            .unwrap()
            .downcast_ref()
            .expect("resource previously loaded as a different type");
    }
    pub fn try_get<T>(&mut self, asset_path: &str) -> Option<&T>
    where
        T: 'static,
    {
        return self.resources.get(asset_path).map(|asset| {
            asset
                .downcast_ref()
                .expect("resource previously loaded as a different type")
        });
    }
    pub fn insert<T>(&mut self, asset_path: &str, asset: T)
    where
        T: 'static,
    {
        if self.resources.contains_key(asset_path) {
            log::warn!(
                "Tried to create resource {} that already exists",
                asset_path
            );
            return;
        }
        log::debug!("Created resource {}", asset_path);
        self.resources
            .insert(asset_path.to_owned(), Box::new(asset));
    }
}
