use crate::render::{
    material::{MaterialCache, MaterialPipeline},
    texture::Texture,
    RenderContext,
};
use bytemuck::{Pod, Zeroable};
use image::{Rgba, RgbaImage};
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, DeviceLocalBuffer},
    descriptor_set::{DescriptorSetWithOffsets, PersistentDescriptorSet, WriteDescriptorSet},
    impl_vertex,
    pipeline::{
        graphics::{
            color_blend::ColorBlendState,
            depth_stencil::DepthStencilState,
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            vertex_input::BuffersDefinition,
            viewport::ViewportState,
        },
        GraphicsPipeline, Pipeline,
    },
    sampler::Sampler,
};

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450
            // vertex
            layout(location = 0) in vec2 position;
            // instance
            layout(location = 1) in vec4 rect;
            layout(location = 2) in vec4 uv_rect;
            layout(location = 3) in vec4 color;
            layout(location = 4) in float depth;

            layout(location = 0) out vec2 v_uv;
            layout(location = 1) out vec4 v_color;

            void main() {
                gl_Position = vec4(rect.xy + (position * rect.zw), depth, 1);
                v_uv = uv_rect.xy + (abs(position) * uv_rect.zw);
                v_color = color;
            }"
    }
}
mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 450
            layout(location = 0) in vec2 v_uv;
            layout(location = 1) in vec4 v_color;

            layout(location = 0) out vec4 f_color;

            layout(set = 0, binding = 0) uniform sampler2D tex;

            void main() {
                f_color = texture(tex, v_uv) * v_color;
            }"
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
    pub position: [f32; 2],
}
impl_vertex!(Vertex, position);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Instance {
    pub rect: [f32; 4],
    pub uv_rect: [f32; 4],
    pub color: [f32; 4],
    pub depth: f32,
}
impl_vertex!(Instance, rect, uv_rect, color, depth);

#[derive(Clone)]
pub struct TextureRectPipeline {
    pipeline: Arc<GraphicsPipeline>,
    pub vertex_buffer: Arc<DeviceLocalBuffer<[Vertex; 4]>>,
    pub none_texture: Texture,
}

impl TextureRectPipeline {
    pub fn new(context: &mut RenderContext) -> Self {
        let vertices = [
            Vertex {
                position: [0.0, 0.0],
            },
            Vertex {
                position: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, 0.0],
            },
            Vertex {
                position: [1.0, 1.0],
            },
        ];
        let allocator = context.allocator().clone();
        let vertex_buffer = DeviceLocalBuffer::from_data(
            &allocator,
            vertices,
            BufferUsage {
                vertex_buffer: true,
                ..BufferUsage::empty()
            },
            context.builder(),
        )
        .unwrap();

        let white_pixel: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
        let none_image = RgbaImage::from_pixel(1, 1, Rgba(white_pixel)).into();
        let none_texture = Texture::load(context, &none_image);

        let vs = vs::load(context.device()).unwrap();
        let fs = fs::load(context.device()).unwrap();

        let subpass = context.render_pass();
        let pipeline = GraphicsPipeline::start()
            .vertex_input_state(
                BuffersDefinition::new()
                    .vertex::<Vertex>()
                    .instance::<Instance>(),
            )
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .input_assembly_state(
                InputAssemblyState::new().topology(PrimitiveTopology::TriangleStrip),
            )
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .color_blend_state(ColorBlendState::new(subpass.num_color_attachments()).blend_alpha())
            .depth_stencil_state(DepthStencilState::simple_depth_test())
            .render_pass(subpass)
            .build(context.device())
            .unwrap();

        TextureRectPipeline {
            pipeline,
            vertex_buffer,
            none_texture,
        }
    }
}

impl MaterialPipeline for TextureRectPipeline {
    fn pipeline(&self) -> &Arc<GraphicsPipeline> {
        &self.pipeline
    }
    fn texture_descriptor_set(
        &self,
        context: &mut RenderContext,
        texture: Texture,
        sampler: Arc<Sampler>,
    ) -> DescriptorSetWithOffsets {
        let layout = self.pipeline.layout().set_layouts().get(0).unwrap().clone();
        PersistentDescriptorSet::new(
            context.descriptor_set_allocator(),
            layout,
            [WriteDescriptorSet::image_view_sampler(
                0,
                texture.image_view().clone(),
                sampler,
            )],
        )
        .unwrap()
        .into()
    }
}

pub type TextureRectRenderer = MaterialCache<TextureRectPipeline, Instance>;
