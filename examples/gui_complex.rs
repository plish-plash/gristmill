use winit::window::Window;
use gristmill::asset::{load_asset, image::NineSliceImage};
use gristmill::game::{Game, run_game};
use gristmill::gui::{Gui, WidgetNode, GuiInputActions, GuiActionEvent, texture_rect::TextureRect, button::ButtonBuilder, text::Text, layout::*};
use gristmill::renderer::{RendererSetup, RendererLoader, RenderPass, pass, subpass};
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

    fn load(&mut self, (_, gui): &mut Scene, renderer_setup: &mut RendererSetup) -> Self::RenderPass {
        let mut render_pass = Self::RenderPass::with_clear_color(renderer_setup, Color::new(0.0, 0.8, 0.8, 1.0));
        let mut gui_subpass_setup = renderer_setup.subpass_setup(render_pass.pass_info(), 1);
        let frame_image: NineSliceImage = load_asset("images/FrameRounded").unwrap();
        let frame_texture = render_pass.subpass1().load_nine_slice_image(&mut gui_subpass_setup, &frame_image);

        let mut layout = Layout::with_base_size(Size { width: 128, height: 128 });
        layout.set_anchor(Side::Top, Anchor::parent(64));
        layout.set_anchor(Side::Left, Anchor::parent(32));
        layout.set_anchor(Side::Right, Anchor::parent(32));
        let texture_rect = gui.add(gui.root(), layout, TextureRect::new(frame_texture));
        
        let mut layout = Layout::with_base_size(Size { width: 128, height: 32 });
        layout.set_anchor(Side::Top, Anchor::parent(16));
        layout.set_anchor(Side::Left, Anchor::parent(16));
        ButtonBuilder::new()
            .with_text("Hello".to_string())
            .build(gui, texture_rect.into(), layout);
        
        let mut layout = Layout::with_base_size(Size { width: 128, height: 32 });
        layout.set_anchor(Side::Top, Anchor { target: AnchorTarget::PreviousSibling, target_side: AnchorTargetSide::OppositeSide, offset: 0 });
        layout.set_anchor(Side::Left, Anchor { target: AnchorTarget::PreviousSibling, target_side: AnchorTargetSide::SameSide, offset: 0 });
        self.text = Some(gui.add(texture_rect.into(), layout, Text::new()));

        render_pass
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

    fn update_renderer(&mut self, _scene: &mut Scene, _render_pass: &mut Self::RenderPass, _loader: &mut RendererLoader) {}

    fn resize(&mut self, _scene: &mut Scene, _dimensions: Size) {}
}

fn main() {
    gristmill::gui::font::load_fonts(vec!["fonts/DejaVuSans".to_string()]); // TODO fonts should be autoloaded
    run_game(GuiGame::new(), ((), Gui::new()))
}
