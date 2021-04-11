use winit::window::Window;
use gristmill::game::{Game, run_game};
use gristmill::gui::{Gui, color_rect::ColorRect, layout::*};
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
        let mut layout = Layout::with_base_size(Size { width: 128, height: 128 });
        layout.set_anchor(Side::Top, Anchor { target: AnchorTarget::Parent, target_side: AnchorTargetSide::SameSide, offset: 64 });
        layout.set_anchor(Side::Left, Anchor { target: AnchorTarget::Parent, target_side: AnchorTargetSide::SameSide, offset: 32 });
        layout.set_anchor(Side::Right, Anchor { target: AnchorTarget::Parent, target_side: AnchorTargetSide::SameSide, offset: 32 });
        gui.set_node_layout(color_rect, layout);

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
