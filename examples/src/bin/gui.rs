use gristmill::{
    geom2d::IRect, geom2d::Size, input::InputActions, math::IVec2, render::RenderContext, run_game,
    Color, Game, GameWindow,
};
use gristmill_gui::{
    widget::{Button, Text, TextAlign, Widget},
    Gui, GuiLayout, GuiRenderer,
};

struct ButtonExample {
    button: Button,
    text: Text,
    times_clicked: usize,
}

impl ButtonExample {
    fn new(gui: &mut Gui) -> Self {
        let root = gui.root();

        let button: Button = gui.create_widget(root.clone(), None);
        button.set_layout(GuiLayout::Child(IRect::new(
            IVec2::new(32, 32),
            Size::new(128, 32),
        )));
        button.set_label_string("Click Me!");

        let text: Text = gui.create_widget(root, None);
        text.set_layout(GuiLayout::Child(IRect::new(
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
    gui: Gui,
    example: ButtonExample,
}

impl GuiGame {
    fn new() -> Self {
        let mut gui = Gui::new();
        let example = ButtonExample::new(&mut gui);
        GuiGame { gui, example }
    }
}

impl Game for GuiGame {
    type Renderer = GuiRenderer;

    fn update(
        &mut self,
        _window: &mut GameWindow,
        input: &InputActions,
        _delta: f64,
    ) -> Option<()> {
        self.gui.update(input);
        self.example.update();
        Some(())
    }

    fn render(&mut self, context: &mut RenderContext, renderer: &mut GuiRenderer) {
        renderer.process(context, &mut self.gui);
        context.begin_render_pass(Color::new(0.0, 0.5, 0.5, 1.0));
        renderer.draw_all(context);
        context.end_render_pass();
    }
}

fn main() {
    run_game(GuiGame::new);
}
