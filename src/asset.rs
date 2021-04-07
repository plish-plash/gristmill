use std::io;
use std::fs::File;
use std::path::PathBuf;

use serde::de::DeserializeOwned;

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
    type AssetStore;
    fn category() -> AssetCategory;
    fn file_extension() -> &'static str;
    fn load(store: Self::AssetStore, file_path: PathBuf) -> io::Result<Self>;
}

pub trait SimpleAsset: Sized {
    fn category() -> AssetCategory;
    fn file_extension() -> &'static str;
    fn load(reader: BufReader) -> io::Result<Self>;
}

impl<T> Asset for T where T: SimpleAsset {
    type AssetStore = ();
    fn category() -> AssetCategory { T::category() }
    fn file_extension() -> &'static str { T::file_extension() }
    fn load(_store: (), file_path: PathBuf) -> io::Result<Self> {
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
    fn load(reader: BufReader) -> io::Result<Self> {
        ron::de::from_reader(reader).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }
}

// -------------------------------------------------------------------------------------------------

// Debug: expect working dir to be cargo project, so look for assets relative to that
#[cfg(debug_assertions)]
fn asset_base_path() -> io::Result<PathBuf> {
    // TODO (???)
    Ok(PathBuf::from("examples"))
    //Ok(PathBuf::new())
}

// Release: always look for assets relative to the executable
#[cfg(not(debug_assertions))]
fn asset_base_path() -> io::Result<PathBuf> {
    // TODO cache
    let mut dir = env::current_exe()?;
    dir.pop();
    Ok(dir)
}

pub fn load_asset<A: Asset>(asset_store: A::AssetStore, asset_path: &str) -> io::Result<A> {
    let mut file_path = asset_base_path()?;
    file_path.push(A::category().file_prefix());
    file_path.push(asset_path);
    file_path.set_extension(A::file_extension());
    A::load(asset_store, file_path)
}
