use std::io;
use std::fs::File;

use serde::de::DeserializeOwned;

// -------------------------------------------------------------------------------------------------

type BufReader = io::BufReader<File>;

pub enum AssetCategory {
    Config, Asset, Resource
}

pub trait Asset: Sized {
    fn category() -> AssetCategory;
    fn file_extension() -> &'static str;
    fn load(reader: BufReader) -> io::Result<Self>;
}

pub trait RonAsset: DeserializeOwned {
    fn category() -> AssetCategory;
}

impl<T> Asset for T where T: RonAsset {
    fn category() -> AssetCategory { T::category() }
    fn file_extension() -> &'static str { ".ron" }
    fn load(reader: BufReader) -> io::Result<Self> {
        ron::de::from_reader(reader).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }
}

// -------------------------------------------------------------------------------------------------

pub fn load_asset<A: Asset>(asset_path: &str) -> io::Result<A> {
    // TODO figure out a base path intelligently instead of just using the working dir
    let file_path = format!("{}{}", asset_path, A::file_extension());
    let reader = BufReader::new(File::open(file_path)?);
    A::load(reader)
}
