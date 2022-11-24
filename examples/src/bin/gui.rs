use gristmill::{
    asset::AssetStorage, geom2d::Rect, geom2d::Size, input::InputActions, math::IVec2,
    render::RenderContext, run_game, Color, Game, GameWindow,
};
use gristmill_gui::{
    widget::{Button, Text, TextAlign, Widget, WidgetStyles},
    Gui, GuiLayout, GuiRenderer,
};

struct Scene {
    gui: Gui,
}

struct ButtonExample {
    button: Button,
    text: Text,
    times_clicked: usize,
}

impl ButtonExample {
    fn new(gui: &mut Gui) -> Self {
        let root = gui.root();

        let button: Button = gui.create_widget(root.clone());
        button.set_layout(GuiLayout::Child(Rect::new(
            IVec2::new(32, 32),
            Size::new(128, 32),
        )));
        button.set_label_string("Click Me!");

        let text: Text = gui.create_widget(root);
        text.set_layout(GuiLayout::Child(Rect::new(
            IVec2::new(32 + 128 + 8, 32),
            Size::new(128, 32),
        )));
        text.set_align(TextAlign::MiddleLeft);

        ButtonExample {
            button,
            text,
            times_clicked: 0,
        }
    }
    fn update(&mut self) {
        if self.button.interact() {
            self.times_clicked += 1;
            self.text
                .set_text_string(format!("Times clicked: {}", self.times_clicked));
        }
    }
}

struct GuiGame {
    scene: Scene,
    example: ButtonExample,
    gui_renderer: GuiRenderer,
}

impl Game for GuiGame {
    fn load(mut config: AssetStorage, context: &mut RenderContext) -> Self {
        let gui_styles = config
            .get_or_save("styles", WidgetStyles::with_all_defaults)
            .clone();
        let mut gui = Gui::with_styles(gui_styles);
        let example = ButtonExample::new(&mut gui);

        GuiGame {
            scene: Scene { gui },
            example,
            gui_renderer: GuiRenderer::new(context),
        }
    }

    fn update(&mut self, _window: &mut GameWindow, input: &InputActions, _delta: f64) {
        self.example.update();
        self.scene.gui.update(input);
    }

    fn render(&mut self, context: &mut RenderContext) {
        self.gui_renderer.pre_render(context, &mut self.scene.gui);
        context.begin_render_pass(Color::new(0.0, 0.5, 0.5, 1.0));
        self.gui_renderer.render(context);
        context.end_render_pass();
    }
}

fn main() {
    run_game::<GuiGame>();
}
