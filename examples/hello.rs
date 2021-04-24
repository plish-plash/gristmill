use winit::window::Window;
use gristmill::game::{Game, run_game};
use gristmill::renderer::{RendererSetup, RendererLoader, pass, subpass};
use gristmill::geometry2d::Size;
use gristmill::input::InputSystem;

// -------------------------------------------------------------------------------------------------

type Scene = ();

struct HelloGame;

impl Game for HelloGame {
    type RenderPass = pass::GeometryPass<subpass::example::ExampleSubpass>;

    fn load(&mut self, _scene: &mut Scene, renderer_setup: &mut RendererSetup) -> Self::RenderPass {
        Self::RenderPass::new(renderer_setup)
    }

    fn update(&mut self, _scene: &mut Scene, _window: &Window, _input_system: &mut InputSystem, _delta: f64) -> bool {
        true
    }

    fn update_renderer(&mut self, _scene: &mut Scene, _render_pass: &mut Self::RenderPass, _loader: &mut RendererLoader) {}

    fn resize(&mut self, _scene: &mut Scene, _dimensions: Size) {}
}

fn main() {
    run_game(HelloGame, ())
}
