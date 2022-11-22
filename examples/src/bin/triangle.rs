use gristmill::{
    asset::Resources, input::InputSystem, render::RenderContext, run_game, Color, Game, Window,
};

use examples::example_renderer::ExampleRenderer;

// -------------------------------------------------------------------------------------------------

struct TriangleGame {
    renderer: ExampleRenderer,
}

impl Game for TriangleGame {
    fn load(_resources: Resources, context: &mut RenderContext) -> Self {
        TriangleGame {
            renderer: ExampleRenderer::new(context),
        }
    }

    fn update(&mut self, _window: &Window, _input_system: &mut InputSystem, _delta: f64) {}

    fn render(&mut self, context: &mut RenderContext) {
        context.begin_render_pass(Color::new(0.0, 0.0, 1.0, 1.0));
        self.renderer.render(context);
        context.end_render_pass();
    }
}

fn main() {
    run_game::<TriangleGame>(Resources::new());
}
