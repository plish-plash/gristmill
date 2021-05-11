pub mod texture;

use crate::asset::resource::AssetItem;
use crate::renderer::{RenderLoader, RenderPass};

pub trait AssetListLoader<'a> {
    type RenderPass: RenderPass + 'a;
    type Output;
    fn name() -> &'static str;
    fn new(loader: &'a mut RenderLoader, render_pass: &'a mut Self::RenderPass) -> Self;
    fn load(&mut self, item: &AssetItem);
    fn finish(self) -> Self::Output;
}
