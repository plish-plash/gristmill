use winit::window::Window;
use gristmill::game::{Game, run_game};
use gristmill::gui::{Gui, WidgetNode, GuiInputActions, GuiActionEvent, color_rect::ColorRect, button::ButtonBuilder, text::Text, layout::*};
use gristmill::renderer::{RendererSetup, RendererLoader, RenderPass, RenderPassInfo, pass, subpass};
use gristmill::color::Color;
use gristmill::geometry2d::*;
use gristmill::input::{InputSystem, InputActions, CursorAction, ActionState};

// -------------------------------------------------------------------------------------------------

type Scene = ((), Gui);

#[derive(Default)]
struct GuiGameInput {
    primary: CursorAction,
}

impl InputActions for GuiGameInput {
    fn end_frame(&mut self) {
        self.primary.end_frame();
    }
    fn set_action_state(&mut self, target: &str, state: ActionState) {
        match target {
            "primary" => self.primary.set_state(state),
            _ => (),
        }
    }
}

impl GuiInputActions for GuiGameInput {
    fn primary(&self) -> &CursorAction { &self.primary }
}

struct GuiGame {
    input: GuiGameInput,
    text: Option<WidgetNode<Text>>,
    times_clicked: u32,
}

impl GuiGame {
    fn new() -> GuiGame {
        GuiGame { input: GuiGameInput::default(), text: None, times_clicked: 0 }
    }
}

impl Game for GuiGame {
    type RenderPass = pass::GeometryGuiPass<subpass::example::ExampleSubpass, subpass::gui::GuiSubpass>;

    fn load(&mut self, (_, gui): &mut Scene, renderer_setup: &mut RendererSetup) -> RenderPassInfo<Self::RenderPass> {
        let mut layout = Layout::with_base_size(Size { width: 128, height: 128 });
        layout.set_anchor(Side::Top, Anchor::parent(64));
        layout.set_anchor(Side::Left, Anchor::parent(32));
        layout.set_anchor(Side::Right, Anchor::parent(32));
        let color_rect = gui.add(gui.root(), layout, ColorRect::new(Color::new(0., 0., 1., 1.)));
        
        let mut layout = Layout::with_base_size(Size { width: 128, height: 32 });
        layout.set_anchor(Side::Top, Anchor::parent(16));
        layout.set_anchor(Side::Left, Anchor::parent(16));
        ButtonBuilder::new()
            .with_text("Hello".to_string())
            .build(gui, color_rect.into(), layout);
        
        let mut layout = Layout::with_base_size(Size { width: 128, height: 32 });
        layout.set_anchor(Side::Top, Anchor { target: AnchorTarget::PreviousSibling, target_side: AnchorTargetSide::OppositeSide, offset: 0 });
        layout.set_anchor(Side::Left, Anchor { target: AnchorTarget::PreviousSibling, target_side: AnchorTargetSide::SameSide, offset: 0 });
        self.text = Some(gui.add(color_rect.into(), layout, Text::new()));

        Self::RenderPass::new(renderer_setup)
    }

    fn update(&mut self, (_, gui): &mut Scene, _window: &Window, input_system: &mut InputSystem, _delta: f64) -> bool {
        input_system.dispatch_queue(&mut self.input);
        let times_clicked = &mut self.times_clicked;
        let old_times_clicked = *times_clicked;
        gui.process_input(&self.input, move |event| {
            match event {
                GuiActionEvent::Action(_) => *times_clicked += 1,
                _ => (),
            }
        });
        if self.times_clicked != old_times_clicked {
            gui.get_mut(self.text.unwrap()).unwrap().set_text(format!("Button clicked {} times.", self.times_clicked));
        }
        true
    }

    fn update_renderer(&mut self, _scene: &mut Scene, _render_pass: &mut RenderPassInfo<Self::RenderPass>, _loader: &mut RendererLoader) {}

    fn resize(&mut self, _scene: &mut Scene, _dimensions: Size) {}
}

fn main() {
    gristmill::gui::font::load_fonts(vec!["fonts/DejaVuSans".to_string()]); // TODO fonts should be autoloaded
    run_game(GuiGame::new(), ((), Gui::new()))
}
