use winit::window::Window;
use gristmill::game::{Game, run_game};
use gristmill::renderer::{RendererSetup, RendererLoader, RenderPass, RenderPassInfo, pass, subpass};
use gristmill::geometry2d::Size;
use gristmill::input::InputSystem;

// -------------------------------------------------------------------------------------------------

type Scene = ();

struct HelloGame;

impl Game for HelloGame {
    type RenderPass = pass::GeometryPass<subpass::example::ExampleSubpass>;

    fn load(&mut self, scene: &mut Scene, renderer_setup: &mut RendererSetup) -> RenderPassInfo<Self::RenderPass> {
        Self::RenderPass::new(renderer_setup)
    }

    fn update(&mut self, scene: &mut Scene, window: &Window, input_system: &mut InputSystem, delta: f64) -> bool {
        true
    }

    fn update_renderer(&mut self, scene: &mut Scene, render_pass: &mut RenderPassInfo<Self::RenderPass>, loader: &mut RendererLoader) {}

    fn resize(&mut self, scene: &mut Scene, dimensions: Size) {}
}

fn main() {
    run_game(HelloGame, ())
}
