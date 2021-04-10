use winit::window::Window;
use gristmill::game::{Game, run_game};
use gristmill::gui::{Gui, color_rect::ColorRect};
use gristmill::renderer::{RendererSetup, RendererLoader, RenderPass, RenderPassInfo, pass, subpass};
use gristmill::color::Color;
use gristmill::geometry2d::*;
use gristmill::input::InputSystem;

// -------------------------------------------------------------------------------------------------

type Scene = ((), Gui);

struct GuiGame;

impl Game for GuiGame {
    type RenderPass = pass::GeometryGuiPass<subpass::example::ExampleSubpass, subpass::gui::GuiSubpass>;

    fn load(&mut self, (_, gui): &mut Scene, renderer_setup: &mut RendererSetup) -> RenderPassInfo<Self::RenderPass> {
        let color_rect = gui.add(gui.root(), ColorRect::new(Color::new(0., 1., 0., 1.)));
        gui.set_node_rect(color_rect, Rect { position: Point::new(64, 64), size: Size::new(256, 64) });

        Self::RenderPass::new(renderer_setup)
    }

    fn update(&mut self, scene: &mut Scene, window: &Window, input_system: &mut InputSystem, delta: f64) -> bool {
        true
    }

    fn update_renderer(&mut self, scene: &mut Scene, render_pass: &mut RenderPassInfo<Self::RenderPass>, loader: &mut RendererLoader) {}

    fn resize(&mut self, scene: &mut Scene, dimensions: Size) {}
}

fn main() {
    run_game(GuiGame, ((), Gui::new()))
}
