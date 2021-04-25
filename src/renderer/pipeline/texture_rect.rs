use std::sync::Arc;

use vulkano::buffer::{ImmutableBuffer, BufferUsage};
use vulkano::command_buffer::{DynamicState, AutoCommandBufferBuilder};
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::descriptor::pipeline_layout::PipelineLayoutAbstract;
use vulkano::format::Format;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::image::{ImmutableImage, ImageDimensions, MipmapsCount};
use vulkano::image::view::ImageView;
use vulkano::sampler::{Sampler, Filter, MipmapMode, SamplerAddressMode};

use crate::asset::image::{Image, NineSliceImage};
use crate::geometry2d::*;
use crate::renderer::{PipelineArc, SubpassSetup};

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
pub struct Texture {
    descriptor_set: Arc<dyn DescriptorSet + Send + Sync>,
}

#[derive(Clone)]
pub struct NineSliceTexture {
    texture: Texture,
    vertex_buffer: Arc<ImmutableBuffer<[Vertex]>>,
}

pub struct TextureRectPipeline {
    pipeline: PipelineArc,
    square_vertex_buffer: Arc<ImmutableBuffer<[Vertex]>>,
}

impl TextureRectPipeline {
    pub fn new(subpass_setup: &mut SubpassSetup) -> TextureRectPipeline {
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

        TextureRectPipeline { pipeline, square_vertex_buffer }
    }

    pub fn draw_rect(&self, builder: &mut AutoCommandBufferBuilder, dynamic_state: &DynamicState, texture: &Texture, push_constants: PushConstants) {
        builder.draw(
            self.pipeline.clone(),
            dynamic_state,
            vec![self.square_vertex_buffer.clone()],
            texture.descriptor_set.clone(),
            push_constants,
            vec![],
        ).unwrap();
    }
    pub fn draw_nine_slice(&self, builder: &mut AutoCommandBufferBuilder, dynamic_state: &DynamicState, texture: &NineSliceTexture, push_constants: PushConstants) {
        builder.draw(
            self.pipeline.clone(),
            dynamic_state,
            vec![texture.vertex_buffer.clone()],
            texture.texture.descriptor_set.clone(),
            push_constants,
            vec![],
        ).unwrap();
    }
    
    pub fn load_image(&mut self, subpass_setup: &mut SubpassSetup, image: &Image, filter: Filter) -> Texture {
        let image_size = image.size();
        let dimensions = ImageDimensions::Dim2d {
            width: image_size.width,
            height: image_size.height,
            array_layers: 1,
        };
        let (image, setup_future): (Arc<ImmutableImage<Format>>, _) = ImmutableImage::from_iter(
            image.data().iter().cloned(),
            dimensions,
            MipmapsCount::One,
            image.format().into(),
            subpass_setup.queue(),
        ).unwrap();
        let image_view = ImageView::new(image.clone()).unwrap();
        subpass_setup.queue_join(setup_future);

        let sampler = Sampler::new(
            subpass_setup.device(),
            filter,
            filter,
            MipmapMode::Nearest,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            0.0, 1.0, 0.0, 0.0
        ).unwrap();

        let descriptor_set = Arc::new(
            PersistentDescriptorSet::start(self.pipeline.descriptor_set_layout(0).unwrap().clone())
                .add_sampled_image(image_view, sampler).unwrap()
                .build().unwrap()
        );
        Texture { descriptor_set }
    }
    pub fn load_nine_slice_image(&mut self, subpass_setup: &mut SubpassSetup, image: &NineSliceImage) -> NineSliceTexture {
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

        let texture = self.load_image(subpass_setup, image.as_image(), Filter::Linear);

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
            subpass_setup.queue(),
        ).unwrap();
        subpass_setup.queue_join(setup_future);
        NineSliceTexture { texture, vertex_buffer }
    }
}
