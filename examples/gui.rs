use winit::window::Window;
use gristmill::game::{Game, run_game};
use gristmill::gui::{Gui, color_rect::ColorRect, text::{Text, Align}, layout::*};
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
        let mut layout = Layout::with_base_size(Size { width: 128, height: 128 });
        layout.set_anchor(Side::Top, Anchor { target: AnchorTarget::Parent, target_side: AnchorTargetSide::SameSide, offset: 64 });
        layout.set_anchor(Side::Left, Anchor { target: AnchorTarget::Parent, target_side: AnchorTargetSide::SameSide, offset: 32 });
        layout.set_anchor(Side::Right, Anchor { target: AnchorTarget::Parent, target_side: AnchorTargetSide::SameSide, offset: 32 });
        let color_rect = gui.add(gui.root(), layout, ColorRect::new(Color::new(0., 1., 0., 1.)));
        let mut text = Text::new();
        text.set_text("Hello".to_string());
        text.set_alignment(Align::Middle, Align::Middle);
        gui.add(color_rect, Layout::fill_parent(0), text);

        Self::RenderPass::new(renderer_setup)
    }

    fn update(&mut self, scene: &mut Scene, window: &Window, input_system: &mut InputSystem, delta: f64) -> bool {
        true
    }

    fn update_renderer(&mut self, scene: &mut Scene, render_pass: &mut RenderPassInfo<Self::RenderPass>, loader: &mut RendererLoader) {}

    fn resize(&mut self, scene: &mut Scene, dimensions: Size) {}
}

fn main() {
    gristmill::gui::font::load_fonts(vec!["fonts/DejaVuSans".to_string()]); // TODO fonts should be autoloaded
    run_game(GuiGame, ((), Gui::new()))
}
