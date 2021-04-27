use vulkano::command_buffer::SubpassContents;

use gristmill::asset::image::{Image, TileAtlasImage};
use gristmill::renderer::{SubpassSetup, RenderContext, pipeline::atlas_rect::{Texture, TileAtlasTexture, AtlasRectPipeline}, subpass};
use gristmill::geometry2d::*;

use super::{Entity, World};

#[derive(Clone)]
pub enum Sprite {
    Texture(Texture),
    Tile(TileAtlasTexture, Index2D),
}

pub struct SpriteSubpass {
    atlas_rect_pipeline: AtlasRectPipeline,
    scale: u32,
}

impl SpriteSubpass {
    pub fn set_scale(&mut self, scale: u32) {
        self.scale = scale;
    }
    
    pub fn load_image(&mut self, subpass_setup: &mut SubpassSetup, image: &Image) -> Sprite {
        Sprite::Texture(self.atlas_rect_pipeline.load_image(subpass_setup, image))
    }
    pub fn load_tile_image(&mut self, subpass_setup: &mut SubpassSetup, image: &TileAtlasImage) -> Sprite {
        Sprite::Tile(self.atlas_rect_pipeline.load_tile_image(subpass_setup, image), Index2D::default())
    }

    fn render_entity(&mut self, context: &mut RenderContext, scene: &World, entity: Entity, parent_position: Point) {
        let obj = scene.forest.get(entity);
        let obj_position = obj.position.offset_from(parent_position);
        if let Some(sprite) = obj.sprite.as_ref() {
            let color = gristmill::color::white();
            match sprite {
                Sprite::Texture(texture) => self.atlas_rect_pipeline.draw_rect_full(context, obj_position, texture, color),
                Sprite::Tile(texture, index) => self.atlas_rect_pipeline.draw_tile(context, obj_position, texture, *index, color),
            }
        }
        for child in scene.forest.iter_children(entity) {
            self.render_entity(context, scene, *child, obj_position);
        }
    }
}

impl subpass::RenderSubpass for SpriteSubpass {
    type SubpassCategory = subpass::Geometry;
    type Scene = World;
    fn contents() -> SubpassContents { SubpassContents::Inline }
    fn new(subpass_setup: &mut SubpassSetup) -> Self {
        let atlas_rect_pipeline = AtlasRectPipeline::new(subpass_setup);
        SpriteSubpass { atlas_rect_pipeline, scale: 1 }
    }
    fn set_dimensions(&mut self, dimensions: Size) {
        let width = dimensions.width as f32 / self.scale as f32;
        let height = dimensions.height as f32 / self.scale as f32;
        self.atlas_rect_pipeline.set_dimensions([width, height]);
    }

    fn pre_render(&mut self, _context: &mut RenderContext, _scene: &mut World) {}

    fn render(&mut self, context: &mut RenderContext, scene: &mut World) {
        self.render_entity(context, scene, scene.render_root, Point::origin());
    }
}
