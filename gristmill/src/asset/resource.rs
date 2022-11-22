use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::asset::{category, Asset, AssetExt, AssetResult};
use crate::impl_ron_asset;

// -------------------------------------------------------------------------------------------------

#[derive(Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Dimensions(HashMap<String, i32>);

impl_ron_asset!(Dimensions, Resource);

#[derive(Deserialize)]
#[serde(from = "(String, String, String)")]
pub struct AssetItem {
    pub name: String,
    pub asset_type: String,
    pub asset_path: String,
}

impl From<(String, String, String)> for AssetItem {
    fn from((name, asset_type, asset_path): (String, String, String)) -> Self {
        AssetItem {
            name,
            asset_type,
            asset_path,
        }
    }
}

#[derive(Default, Deserialize)]
pub struct AssetList {
    #[serde(skip)]
    name: String,
    loader: String,
    assets: Vec<AssetItem>,
}

impl Asset for AssetList {
    type Category = category::Resource;
    fn read(asset_path: &str) -> AssetResult<Self> {
        let mut list: AssetList = Self::read_ron(asset_path)?;
        list.name = asset_path.to_owned();
        Ok(list)
    }
}

impl AssetList {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn loader(&self) -> &str {
        &self.loader
    }
    pub fn iter(&self) -> std::slice::Iter<AssetItem> {
        self.assets.iter()
    }
}

impl IntoIterator for AssetList {
    type Item = AssetItem;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.assets.into_iter()
    }
}
