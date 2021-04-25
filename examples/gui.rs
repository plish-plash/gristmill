use winit::window::Window;
use gristmill::game::{Game, run_game};
use gristmill::gui::{Gui, WidgetNode, GuiInputActions, GuiActionEvent, color_rect::ColorRect, button::ButtonBuilder, text::Text, layout::*};
use gristmill::renderer::{RenderPassInfo, Renderer, RenderContext, pass::{RenderPass, GeometryGuiPass}, subpass};
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
    render_pass: GeometryGuiPass<subpass::example::ExampleSubpass, subpass::gui::GuiSubpass>,
    scene: Scene,
    input: GuiGameInput,
    text: WidgetNode<Text>,
    times_clicked: u32,
}

impl Game for GuiGame {
    fn load(renderer: &mut Renderer) -> (Self, RenderPassInfo) {
        let mut gui = Gui::new();

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
            .build(&mut gui, color_rect.into(), layout);
        
        let mut layout = Layout::with_base_size(Size { width: 128, height: 32 });
        layout.set_anchor(Side::Top, Anchor { target: AnchorTarget::PreviousSibling, target_side: AnchorTargetSide::OppositeSide, offset: 0 });
        layout.set_anchor(Side::Left, Anchor { target: AnchorTarget::PreviousSibling, target_side: AnchorTargetSide::SameSide, offset: 0 });
        let text = gui.add(color_rect.into(), layout, Text::new());

        let render_pass = GeometryGuiPass::new(renderer);
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
    gristmill::gui::font::load_fonts(vec!["fonts/DejaVuSans".to_string()]); // TODO fonts should be autoloaded
    run_game::<GuiGame>();
}
