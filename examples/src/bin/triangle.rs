use bytemuck::{Pod, Zeroable};
use gristmill::{
    input::InputSystem,
    render::Renderable,
    {render::RenderContext, run_game, Game, GameWindow},
};
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
    pipeline::{
        graphics::input_assembly::InputAssemblyState, graphics::vertex_input::BuffersDefinition,
        graphics::viewport::ViewportState, GraphicsPipeline,
    },
};

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

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
    pub position: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position);

pub struct ExamplePipeline(Arc<GraphicsPipeline>);

impl ExamplePipeline {
    pub fn new(context: &mut RenderContext) -> ExamplePipeline {
        let vs = vs::load(context.device()).unwrap();
        let fs = fs::load(context.device()).unwrap();
        ExamplePipeline(
            GraphicsPipeline::start()
                .render_pass(context.render_pass())
                .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
                .input_assembly_state(InputAssemblyState::new())
                .vertex_shader(vs.entry_point("main").unwrap(), ())
                .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
                .fragment_shader(fs.entry_point("main").unwrap(), ())
                .build(context.device())
                .unwrap(),
        )
    }
    pub fn bind(&self, context: &mut RenderContext) {
        context.builder().bind_pipeline_graphics(self.0.clone());
    }
}

pub struct ExampleRenderer {
    pipeline: ExamplePipeline,
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
}

impl ExampleRenderer {
    pub fn new(context: &mut RenderContext) -> Self {
        let pipeline = ExamplePipeline::new(context);

        let vertices = [
            Vertex {
                position: [-0.5, -0.25],
            },
            Vertex {
                position: [0.0, 0.5],
            },
            Vertex {
                position: [0.25, -0.1],
            },
        ];
        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            context.allocator(),
            BufferUsage {
                vertex_buffer: true,
                ..BufferUsage::empty()
            },
            false,
            vertices,
        )
        .unwrap();

        ExampleRenderer {
            pipeline,
            vertex_buffer,
        }
    }
    pub fn render(&self, context: &mut RenderContext) {
        self.pipeline.bind(context);
        context
            .builder()
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .draw(self.vertex_buffer.len() as u32, 1, 0, 0)
            .unwrap();
    }
}

struct TriangleGame {
    input_system: InputSystem,
    renderer: ExampleRenderer,
}

impl TriangleGame {
    fn new(context: &mut RenderContext) -> Self {
        TriangleGame {
            input_system: InputSystem::load_config(),
            renderer: ExampleRenderer::new(context),
        }
    }
}

impl Renderable for TriangleGame {
    fn pre_render(&mut self, _context: &mut RenderContext) {}
    fn render(&mut self, context: &mut RenderContext) {
        self.renderer.render(context);
    }
}

impl Game for TriangleGame {
    fn input_system(&mut self) -> &mut InputSystem {
        &mut self.input_system
    }
    fn update(&mut self, window: &mut GameWindow, _delta: f64) {
        let input_actions = self.input_system.actions();
        if input_actions.get("exit").just_pressed() {
            window.close();
        }
    }
}

fn main() {
    run_game(TriangleGame::new);
}
