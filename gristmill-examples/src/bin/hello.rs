use gristmill::game::{Game, Window, run_game};
use gristmill::renderer::{RenderLoader, RenderContext, pass::{RenderPass, RenderPass3D}};
use gristmill::input::InputSystem;

use gristmill_examples::basic_geo_renderer::BasicGeoRenderer;

// -------------------------------------------------------------------------------------------------

struct HelloGame;

impl Game for HelloGame {
    type RenderPass = RenderPass3D<BasicGeoRenderer>;
    fn load(loader: &mut RenderLoader) -> (Self, Self::RenderPass) {
        (HelloGame, RenderPass3D::new(loader))
    }
    fn update(&mut self, _window: &Window, _input_system: &mut InputSystem, _delta: f64) -> bool {
        true
    }
    fn render(&mut self, _loader: &mut RenderLoader, context: &mut RenderContext, render_pass: &mut Self::RenderPass) {
        render_pass.render(context, &mut ());
    }
}

fn main() {
    run_game::<HelloGame>();
}
