use gristmill::asset::Resources;
use gristmill::game::{Game, Window, run_game};
use gristmill_gui::{Gui, WidgetNode, GuiInputActions, event::GuiActionEvent, quad::Quad, button::ButtonClass, text::Text, layout::*};
use gristmill::renderer::{RenderLoader, RenderContext, pass::{RenderPass, RenderPass3D2D}};
use gristmill::color::Color;
use gristmill::geometry2d::*;
use gristmill::input::{InputSystem, InputActions, CursorAction, ActionState};

use gristmill_examples::basic_geo_renderer::BasicGeoRenderer;
use gristmill_gui::renderer::GuiRenderer;

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
    scene: Scene,
    input: GuiGameInput,
    text: WidgetNode<Text>,
    times_clicked: u32,
}

impl Game for GuiGame {
    type RenderPass = RenderPass3D2D<BasicGeoRenderer, GuiRenderer>;
    fn load(_resources: Resources, loader: &mut RenderLoader) -> (Self, Self::RenderPass) {
        let mut gui = Gui::new();
        gui.set_event_handler(gui.root());

        let mut layout = Layout::new_size(Size::new(128, 128));
        layout.set_anchor(Side::Top, Anchor::parent(64));
        layout.set_anchor(Side::Left, Anchor::parent(32));
        layout.set_anchor(Side::Right, Anchor::parent(32));
        let color_rect = gui.add_widget(gui.root(), layout, Quad::new_color(Color::new(0., 0., 1., 1.)));
        
        let mut layout = Layout::new_size(Size::new(128, 32));
        layout.set_anchor(Side::Top, Anchor::parent(16));
        layout.set_anchor(Side::Left, Anchor::parent(16));
        ButtonClass::new().instance_builder()
            .with_layout(layout)
            .with_text("Hello".to_string())
            .build(&mut gui, color_rect.into());
        
        let mut layout = Layout::new_size(Size::new(128, 32));
        layout.set_anchor(Side::Top, Anchor::previous_sibling_opposite(16));
        layout.set_anchor(Side::Left, Anchor::previous_sibling(0));
        let text = gui.add_widget(color_rect.into(), layout, Text::new_empty());

        (GuiGame {
            scene: ((), gui),
            input: GuiGameInput::default(),
            text,
            times_clicked: 0,
        }, RenderPass3D2D::new(loader))
    }

    fn update(&mut self, _window: &Window, input_system: &mut InputSystem, _delta: f64) -> bool {
        input_system.dispatch_queue(&mut self.input);
        let gui = &mut self.scene.1;
        gui.process_input(&self.input);

        let times_clicked = &mut self.times_clicked;
        let old_times_clicked = *times_clicked;
        gui.get_events(gui.root()).unwrap().dispatch_queue(move |event| {
            match event {
                GuiActionEvent::Generic => *times_clicked += 1,
                _ => (),
            }
        });
        if self.times_clicked != old_times_clicked {
            gui.get_mut(self.text).unwrap().set_text(format!("Button clicked {} times.", self.times_clicked));
        }
        true
    }

    fn render(&mut self, _loader: &mut RenderLoader, context: &mut RenderContext, render_pass: &mut Self::RenderPass) {
        render_pass.render(context, &mut self.scene);
    }
}

fn main() {
    let mut resources = Resources::new();
    gristmill_gui::font::load_fonts(&mut resources);
    run_game::<GuiGame>(resources);
}
