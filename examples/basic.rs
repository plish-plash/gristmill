use winit::window::Window;
use gristmill::game::{Game, run_game};
use gristmill::renderer::{RendererSetup, RendererLoader, RenderPass, RenderPassInfo, pass, subpass};
use gristmill::geometry2d::Size;
use gristmill::input::{InputActions, ActionState, Axis2};

// -------------------------------------------------------------------------------------------------

type Scene = ();

struct BasicGame;

#[derive(Default)]
struct EmptyInputActions;

impl InputActions for EmptyInputActions {
    fn end_frame(&mut self) {}
    fn set_action_state_button(&mut self, _target: &str, _state: ActionState<bool>) {}
    fn set_action_state_axis1(&mut self, _target: &str, _state: ActionState<f32>) {}
    fn set_action_state_axis2(&mut self, _target: &str, _state: ActionState<Axis2>) {}
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
