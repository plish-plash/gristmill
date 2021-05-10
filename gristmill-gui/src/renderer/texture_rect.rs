use std::sync::Arc;

use vulkano::buffer::{ImmutableBuffer, BufferUsage};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::sampler::Filter;

use gristmill::asset::image::{Image, NineSliceImage};
use gristmill::geometry2d::*;
use gristmill::renderer::{PipelineArc, LoadContext, RenderContext};

use gristmill::renderer::loader::texture::TextureLoader;
pub use gristmill::renderer::loader::texture::Texture;

mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        src: "
            #version 450

            layout(push_constant) uniform PushConstants {
                vec2 screen_size;
                vec2 position;
                vec2 size;
                vec4 color;
            } constants;

            layout(location = 0) in vec2 position;
            layout(location = 1) in vec2 tex_position;
            layout(location = 0) out vec2 v_tex_position;
            layout(location = 1) out vec4 v_color;

            void main() {
                vec2 inner_position = clamp(position, 0, 1);
                vec2 offset = position - inner_position;
                vec2 normalized_position = (constants.position + (inner_position * constants.size) + offset) / constants.screen_size;
                gl_Position = vec4((normalized_position - 0.5) * 2.0, 0.0, 1.0);
                v_tex_position = tex_position;
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

pub use vs::ty::PushConstants;

#[derive(Copy, Clone, Default, Debug)]
struct Vertex {
    position: [f32; 2],
    tex_position: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position, tex_position);

impl Vertex {
    fn new(x: f32, y: f32) -> Vertex {
        Vertex {
            position: [x, y],
            tex_position: [x, y],
        }
    }
    fn from_xy(x: Vertex, y: Vertex) -> Vertex {
        Vertex {
            position: [x.position[0], y.position[1]],
            tex_position: [x.tex_position[0], y.tex_position[1]],
        }
    }
}

#[derive(Clone)]
pub struct NineSliceTexture {
    texture: Texture,
    slices: EdgeRect,
    vertex_buffer: Arc<ImmutableBuffer<[Vertex]>>,
}

impl NineSliceTexture {
    pub fn slices(&self) -> EdgeRect { self.slices }
    pub fn as_texture(&self) -> &Texture { &self.texture }
}

pub struct TextureRectPipeline {
    pipeline: PipelineArc,
    square_vertex_buffer: Arc<ImmutableBuffer<[Vertex]>>,
}

impl TextureRectPipeline {
    pub fn new(context: &mut LoadContext) -> TextureRectPipeline {
        let vs = vs::Shader::load(context.device()).unwrap();
        let fs = fs::Shader::load(context.device()).unwrap();

        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .blend_alpha_blending()
                .render_pass(context.subpass())
                .build(context.device())
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
            context.queue(),
        ).unwrap();
        context.load_future(setup_future);

        TextureRectPipeline { pipeline, square_vertex_buffer }
    }

    pub fn draw_rect(&self, context: &mut RenderContext, texture: &Texture, push_constants: PushConstants) {
        context.draw(
            self.pipeline.clone(),
            vec![self.square_vertex_buffer.clone()],
            texture.descriptor_set(),
            push_constants
        );
    }
    pub fn draw_nine_slice(&self, context: &mut RenderContext, texture: &NineSliceTexture, push_constants: PushConstants) {
        context.draw(
            self.pipeline.clone(),
            vec![texture.vertex_buffer.clone()],
            texture.texture.descriptor_set(),
            push_constants
        );
    }
    
    pub fn load_image(&mut self, context: &mut LoadContext, image: &Image, filter: Filter) -> Texture {
        TextureLoader::load_image(&self.pipeline, context, image, filter)
    }
    pub fn load_nine_slice_image(&mut self, context: &mut LoadContext, image: &NineSliceImage) -> NineSliceTexture {
        fn vertex_quad(vertices: &mut Vec<Vertex>, corners: &[Vertex; 4], indices: [usize; 2]) {
            let min = Vertex::from_xy(corners[indices[0]], corners[indices[1]]);
            let max = Vertex::from_xy(corners[indices[0] + 1], corners[indices[1] + 1]);
            vertices.push(Vertex::from_xy(min, max));
            vertices.push(Vertex::from_xy(min, min));
            vertices.push(Vertex::from_xy(max, min));
            vertices.push(Vertex::from_xy(max, min));
            vertices.push(Vertex::from_xy(max, max));
            vertices.push(Vertex::from_xy(min, max));
        }

        let texture = self.load_image(context, image.as_image(), Filter::Linear);

        let size = image.size();
        let slices = image.slices();
        let inner_rect = Rect { position: Point::origin(), size }.inset(slices);
        let corners = [
            Vertex { position: Point::new(-slices.left, -slices.top).into(), tex_position: [0., 0.] },
            Vertex { position: [0., 0.], tex_position: inner_rect.top_left().normalize_components(size) },
            Vertex { position: [1., 1.], tex_position: inner_rect.bottom_right().normalize_components(size) },
            Vertex { position: Point::new(1 + slices.right, 1 + slices.bottom).into(), tex_position: [1., 1.] },
        ];
        let mut verts = Vec::with_capacity(9 * 6);
        vertex_quad(&mut verts, &corners, [0, 0]); vertex_quad(&mut verts, &corners, [1, 0]); vertex_quad(&mut verts, &corners, [2, 0]);
        vertex_quad(&mut verts, &corners, [0, 1]); vertex_quad(&mut verts, &corners, [1, 1]); vertex_quad(&mut verts, &corners, [2, 1]);
        vertex_quad(&mut verts, &corners, [0, 2]); vertex_quad(&mut verts, &corners, [1, 2]); vertex_quad(&mut verts, &corners, [2, 2]);
        let (vertex_buffer, setup_future) = ImmutableBuffer::from_iter(
            verts.into_iter(),
            BufferUsage::vertex_buffer(),
            context.queue(),
        ).unwrap();
        context.load_future(setup_future);
        NineSliceTexture { texture, slices, vertex_buffer }
    }
}
