use gristmill::game::{Game, Window, run_game};
use gristmill::renderer::{RenderPassInfo, Renderer, RenderContext, pass::{RenderPass, GeometryPass}};
use gristmill::geometry2d::Size;
use gristmill::input::InputSystem;

use gristmill_examples::basic_geo_subpass::BasicGeoSubpass;

// -------------------------------------------------------------------------------------------------

struct HelloGame {
    render_pass: GeometryPass<BasicGeoSubpass>
}

impl Game for HelloGame {
    fn load(renderer: &mut Renderer) -> (Self, RenderPassInfo) {
        let render_pass = GeometryPass::new(renderer);
        let render_pass_info = render_pass.info();
        (HelloGame { render_pass }, render_pass_info)
    }
    fn resize(&mut self, dimensions: Size) {
        self.render_pass.set_dimensions(dimensions);
    }
    fn update(&mut self, _window: &Window, _input_system: &mut InputSystem, _delta: f64) -> bool {
        true
    }
    fn render(&mut self, _renderer: &mut Renderer, context: &mut RenderContext) {
        self.render_pass.render(context, &mut ());
    }
}

fn main() {
    run_game::<HelloGame>();
}
