use winit::window::Window;
use gristmill::renderer::{Game, RendererSetup, RendererLoader, pass::{self, RenderPass, RenderPassInfo}, subpass, run_game};
use gristmill::gui::geometry::Size;
use gristmill::input::{InputActions, ActionState, Axis2};

// ------------------------------------------------------------------------------------------------

type Scene = ();

struct BasicGame;

#[derive(Default)]
struct EmptyInputActions;

impl InputActions for EmptyInputActions {
    fn end_frame(&mut self) {}
    fn set_action_state_button(&mut self, target: &str, state: ActionState<bool>) {}
    fn set_action_state_axis1(&mut self, target: &str, _state: ActionState<f32>) {}
    fn set_action_state_axis2(&mut self, target: &str, state: ActionState<Axis2>) {}
}

impl Game for BasicGame {
    type RenderPass = pass::GeometryPass<subpass::example::ExampleSubpass>;
    type InputActions = EmptyInputActions;

    fn load(&mut self, scene: &mut Scene, renderer_setup: &mut RendererSetup) -> RenderPassInfo<Self::RenderPass> {
        Self::RenderPass::new(renderer_setup)
    }

    fn update(&mut self, scene: &mut Scene, window: &Window, input: &EmptyInputActions, delta: f64) -> bool {
        true
    }

    fn update_renderer(&mut self, scene: &mut Scene, render_pass: &mut RenderPassInfo<Self::RenderPass>, loader: &mut RendererLoader) {}

    fn resize(&mut self, scene: &mut Scene, dimensions: Size) {}
}

fn main() {
    run_game(BasicGame, ())
}
