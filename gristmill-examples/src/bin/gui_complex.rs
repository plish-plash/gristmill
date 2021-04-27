use gristmill::asset::{load_asset, image::NineSliceImage};
use gristmill::game::{Game, Window, run_game};
use gristmill_gui::{Gui, WidgetNode, GuiInputActions, GuiActionEvent, texture_rect::TextureRect, button::ButtonBuilder, text::Text, layout::*};
use gristmill::renderer::{RenderPassInfo, Renderer, RenderContext, pass::{RenderPass, GeometryGuiPass}};
use gristmill::color::Color;
use gristmill::geometry2d::*;
use gristmill::input::{InputSystem, InputActions, CursorAction, ActionState};

use gristmill_examples::basic_geo_subpass::BasicGeoSubpass;
use gristmill_gui::renderer::GuiSubpass;

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
    render_pass: GeometryGuiPass<BasicGeoSubpass, GuiSubpass>,
    scene: Scene,
    input: GuiGameInput,
    text: WidgetNode<Text>,
    times_clicked: u32,
}

impl Game for GuiGame {
    fn load(renderer: &mut Renderer) -> (Self, RenderPassInfo) {
        let mut render_pass = GeometryGuiPass::<BasicGeoSubpass, GuiSubpass>::with_clear_color(renderer, Color::new(0.0, 0.8, 0.8, 1.0));
        let mut gui_subpass_setup = renderer.subpass_setup(render_pass.info(), 1);
        let frame_image: NineSliceImage = load_asset("images/FrameRounded").unwrap();
        let frame_texture = render_pass.subpass1().load_nine_slice_image(&mut gui_subpass_setup, &frame_image);

        let mut gui = Gui::new();

        let mut layout = Layout::with_base_size(Size { width: 128, height: 128 });
        layout.set_anchor(Side::Top, Anchor::parent(64));
        layout.set_anchor(Side::Left, Anchor::parent(32));
        layout.set_anchor(Side::Right, Anchor::parent(32));
        let color_rect = gui.add(gui.root(), layout, TextureRect::new(frame_texture));
        
        let mut layout = Layout::with_base_size(Size { width: 128, height: 32 });
        layout.set_anchor(Side::Top, Anchor::parent(16));
        layout.set_anchor(Side::Left, Anchor::parent(16));
        ButtonBuilder::new()
            .with_text("Hello".to_string())
            .build(&mut gui, color_rect.into(), layout);
        
        let mut layout = Layout::with_base_size(Size { width: 128, height: 32 });
        layout.set_anchor(Side::Top, Anchor { target: AnchorTarget::PreviousSibling, target_side: AnchorTargetSide::OppositeSide, offset: 0 });
        layout.set_anchor(Side::Left, Anchor { target: AnchorTarget::PreviousSibling, target_side: AnchorTargetSide::SameSide, offset: 0 });
        let text = gui.add(color_rect.into(), layout, Text::new());

        let render_pass_info = render_pass.info();
        (GuiGame {
            render_pass,
            scene: ((), gui),
            input: GuiGameInput::default(),
            text,
            times_clicked: 0,
        }, render_pass_info)
    }

    fn resize(&mut self, dimensions: Size) {
        self.render_pass.set_dimensions(dimensions);
    }

    fn update(&mut self, _window: &Window, input_system: &mut InputSystem, _delta: f64) -> bool {
        input_system.dispatch_queue(&mut self.input);
        let gui = &mut self.scene.1;
        let times_clicked = &mut self.times_clicked;
        let old_times_clicked = *times_clicked;
        gui.process_input(&self.input, move |event| {
            match event {
                GuiActionEvent::Action(_) => *times_clicked += 1,
                _ => (),
            }
        });
        if self.times_clicked != old_times_clicked {
            gui.get_mut(self.text).unwrap().set_text(format!("Button clicked {} times.", self.times_clicked));
        }
        true
    }

    fn render(&mut self, _renderer: &mut Renderer, context: &mut RenderContext) {
        self.render_pass.render(context, &mut self.scene);
    }
}

fn main() {
    gristmill_gui::font::load_fonts(vec!["fonts/DejaVuSans".to_string()]); // TODO fonts should be autoloaded
    run_game::<GuiGame>();
}
