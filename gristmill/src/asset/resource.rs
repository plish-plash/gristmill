use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::impl_ron_asset;

// -------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
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
        AssetItem { name, asset_type, asset_path }
    }
}

#[derive(Deserialize)]
pub struct AssetList {
    loader: String,
    assets: Vec<AssetItem>,
}

impl_ron_asset!(AssetList, Resource);

impl AssetList {
    pub fn loader(&self) -> &str { &self.loader }
    pub fn iter(&self) -> std::slice::Iter<AssetItem> { self.assets.iter() }
}

impl IntoIterator for AssetList {
    type Item = AssetItem;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.assets.into_iter()
    }
}
