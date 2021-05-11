pub mod texture;

use crate::renderer::{RenderAsset, LoadContext};

pub trait RenderAssetLoader {
    type RenderAsset: RenderAsset;
    fn name() -> &'static str;
    fn load(&mut self, context: &mut LoadContext, asset_type: &str, asset_path: &str) -> Option<Self::RenderAsset>;
}
