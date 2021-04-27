use std::sync::Arc;

use vulkano::command_buffer::SubpassContents;
use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};

use gristmill::renderer::{SubpassSetup, RenderContext, subpass};

// -------------------------------------------------------------------------------------------------
// This is a pipeline and subpass that just draws a static red triangle. Useful for testing.

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450

            layout(location = 0) in vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        "
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 450

            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        "
    }
}

#[derive(Default, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position);

pub struct BasicGeoPipeline {
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
}

impl BasicGeoPipeline {
    pub fn new(subpass_setup: &mut SubpassSetup) -> BasicGeoPipeline {
        let vs = vs::Shader::load(subpass_setup.device()).unwrap();
        let fs = fs::Shader::load(subpass_setup.device()).unwrap();
    
        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                //.cull_mode_front()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                //.depth_stencil_simple_depth()
                .render_pass(subpass_setup.subpass())
                .build(subpass_setup.device())
                .unwrap(),
        );
    
        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            subpass_setup.device(),
            BufferUsage::all(),
            false,
            [
                Vertex { position: [-0.5, -0.25] },
                Vertex { position: [0.0, 0.5] },
                Vertex { position: [0.25, -0.1] },
            ].iter().cloned(),
        ).unwrap();

        BasicGeoPipeline { pipeline, vertex_buffer }
    }
}

// -------------------------------------------------------------------------------------------------

pub struct BasicGeoSubpass(BasicGeoPipeline);

impl subpass::RenderSubpass for BasicGeoSubpass {
    type SubpassCategory = subpass::Geometry;
    type Scene = ();
    fn contents() -> SubpassContents { SubpassContents::Inline }
    fn new(subpass_setup: &mut SubpassSetup) -> Self {
        BasicGeoSubpass(BasicGeoPipeline::new(subpass_setup))
    }
    fn pre_render(&mut self, _context: &mut RenderContext, _scene: &mut Self::Scene) {}
    fn render(&mut self, context: &mut RenderContext, _scene: &mut Self::Scene) {
        context.draw(
            self.0.pipeline.clone(),
            vec![self.0.vertex_buffer.clone()],
            (),
            ()
        );
    }
}
