use std::sync::Arc;

use vulkano::buffer::{ImmutableBuffer, BufferUsage};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::sampler::Filter;

use crate::asset::image::{Image, TileAtlasImage};
use crate::color::{Color, encode_color};
use crate::geometry2d::*;
use crate::renderer::{PipelineArc, SubpassSetup, RenderContext};

use super::texture::TexturePipeline;
pub use super::texture::Texture;

mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        src: "
            #version 450

            layout(push_constant) uniform PushConstants {
                vec2 screen_size;
                vec4 rect;
                vec4 tex_rect;
                vec4 color;
            } constants;

            layout(location = 0) in vec2 position;
            layout(location = 0) out vec2 v_tex_position;
            layout(location = 1) out vec4 v_color;

            void main() {
                vec2 normalized_position = (constants.rect.xy + (position * constants.rect.zw)) / constants.screen_size;
                gl_Position = vec4(normalized_position * 2.0, 0.0, 1.0);
                v_tex_position = constants.tex_rect.xy + (position * constants.tex_rect.zw);
                v_color = constants.color;
            }
        "
    }
}

mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        src: "
            #version 450

            layout(location = 0) in vec2 v_tex_position;
            layout(location = 1) in vec4 v_color;
            layout(location = 0) out vec4 f_color;

            layout(set = 0, binding = 0) uniform sampler2D tex;

            void main() {
                f_color = v_color * texture(tex, v_tex_position);
            }
        "
    }
}

#[derive(Copy, Clone, Default, Debug)]
struct Vertex {
    position: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position);

impl Vertex {
    fn new(x: f32, y: f32) -> Vertex {
        Vertex {
            position: [x, y],
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TextureRegion([f32; 4]);

impl TextureRegion {
    fn full() -> TextureRegion { TextureRegion([0., 0., 1., 1.]) }
    fn from_rect(texture: &Texture, pixel_rect: Rect) -> TextureRegion {
        let width = texture.size().width as f32;
        let height = texture.size().height as f32;
        TextureRegion([
            (pixel_rect.position.x as f32) / width,
            (pixel_rect.position.y as f32) / height,
            (pixel_rect.size.width as f32) / width,
            (pixel_rect.size.height as f32) / height,
        ])
    }
    fn size_in_pixels(&self, texture: &Texture) -> Size {
        let width = (self.0[2] * (texture.size().width as f32)).round() as u32;
        let height = (self.0[3] * (texture.size().height as f32)).round() as u32;
        Size { width, height }
    }
}

#[derive(Clone)]
pub struct TileAtlasTexture {
    texture: Texture,
    tile_size: Size,
    tile_offset: Point,
    tile_gap: Point,
}

impl TileAtlasTexture {
    pub fn get_tile(&self, tile_index: Index2D) -> TextureRegion {
        let x = self.tile_offset.x + (tile_index.col as i32 * (self.tile_size.width as i32 + self.tile_gap.x));
        let y = self.tile_offset.y + (tile_index.row as i32 * (self.tile_size.height as i32 + self.tile_gap.y));
        TextureRegion::from_rect(&self.texture, Rect { position: Point::new(x, y), size: self.tile_size })
    }
}

pub struct AtlasRectPipeline {
    pipeline: PipelineArc,
    square_vertex_buffer: Arc<ImmutableBuffer<[Vertex]>>,
    screen_dimensions: [f32; 2],
}

impl AtlasRectPipeline {
    pub fn new(subpass_setup: &mut SubpassSetup) -> AtlasRectPipeline {
        let vs = vs::Shader::load(subpass_setup.device()).unwrap();
        let fs = fs::Shader::load(subpass_setup.device()).unwrap();

        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .blend_alpha_blending()
                .render_pass(subpass_setup.subpass())
                .build(subpass_setup.device())
                .unwrap()
        );

        let (square_vertex_buffer, setup_future) = ImmutableBuffer::from_iter(
            vec![
                Vertex::new(0., 1.),
                Vertex::new(0., 0.),
                Vertex::new(1., 0.),
                Vertex::new(1., 0.),
                Vertex::new(1., 1.),
                Vertex::new(0., 1.),
            ].into_iter(),
            BufferUsage::vertex_buffer(),
            subpass_setup.queue(),
        ).unwrap();
        subpass_setup.queue_join(setup_future);

        AtlasRectPipeline { pipeline, square_vertex_buffer, screen_dimensions: [0., 0.] }
    }

    pub fn set_dimensions(&mut self, dimensions: [f32; 2]) {
        self.screen_dimensions = dimensions;
    }

    pub fn draw_rect(&self, context: &mut RenderContext, position: Point, texture: &Texture, region: TextureRegion, color: Color) {
        let push_constants = vs::ty::PushConstants {
            screen_size: self.screen_dimensions,
            rect: Rect { position, size: region.size_in_pixels(texture) }.into(),
            tex_rect: region.0,
            color: encode_color(color),
            _dummy0: [0; 8],
        };
        context.draw(
            self.pipeline.clone(),
            vec![self.square_vertex_buffer.clone()],
            texture.descriptor_set.clone(),
            push_constants
        );
    }
    pub fn draw_rect_full(&self, context: &mut RenderContext, position: Point, texture: &Texture, color: Color) {
        self.draw_rect(context, position, texture, TextureRegion::full(), color);
    }
    pub fn draw_tile(&self, context: &mut RenderContext, position: Point, texture: &TileAtlasTexture, tile_index: Index2D, color: Color) {
        let region = texture.get_tile(tile_index);
        self.draw_rect(context, position, &texture.texture, region, color);
    }
    
    pub fn load_image(&mut self, subpass_setup: &mut SubpassSetup, image: &Image) -> Texture {
        TexturePipeline::load_image(&self.pipeline, subpass_setup, image, Filter::Nearest)
    }
    pub fn load_tile_image(&mut self, subpass_setup: &mut SubpassSetup, image: &TileAtlasImage) -> TileAtlasTexture {
        let texture = TexturePipeline::load_image(&self.pipeline, subpass_setup, image.as_image(), Filter::Nearest);
        TileAtlasTexture {
            texture,
            tile_size: image.tile_size(),
            tile_offset: image.tile_offset(),
            tile_gap: image.tile_gap(),
        }
    }
}
