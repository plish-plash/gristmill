use gristmill::{
    geom2d::{IRect, Size},
    gui::{
        widget::{Button, Text, TextAlign, WidgetNode},
        Gui, GuiLayout,
    },
    input::InputSystem,
    math::IVec2,
    render::{RenderContext, Renderable},
    run_game, Game, GameWindow,
};

struct ButtonExample {
    button: Button,
    text: Text,
    times_clicked: usize,
}

impl ButtonExample {
    fn new(gui: &mut Gui) -> Self {
        let root = gui.root();

        let button: Button = gui.create_widget(root, None);
        button.set_layout(
            gui,
            GuiLayout::Child(IRect::new(IVec2::new(32, 32), Size::new(128, 32))),
        );
        button.set_label_string(gui, "Click Me!");

        let text: Text = gui.create_widget(root, None);
        text.set_layout(
            gui,
            GuiLayout::Child(IRect::new(IVec2::new(32 + 128 + 8, 32), Size::new(128, 32))),
        );
        text.set_align(gui, TextAlign::MiddleLeft);

        ButtonExample {
            button,
            text,
            times_clicked: 0,
        }
    }
    fn update(&mut self, gui: &mut Gui) {
        if self.button.interact() {
            self.times_clicked += 1;
            self.text
                .set_text_string(gui, format!("Times clicked: {}", self.times_clicked));
        }
    }
}

struct GuiGame {
    input_system: InputSystem,
    gui: Gui,
    example: ButtonExample,
}

impl GuiGame {
    fn new(context: &mut RenderContext) -> Self {
        let mut gui = Gui::new(context);
        let example = ButtonExample::new(&mut gui);
        GuiGame {
            input_system: InputSystem::load_bindings(),
            gui,
            example,
        }
    }
}

impl Renderable for GuiGame {
    fn pre_render(&mut self, context: &mut RenderContext) {
        self.gui.pre_render(context);
    }
    fn render(&mut self, context: &mut RenderContext) {
        self.gui.render(context);
    }
}

impl Game for GuiGame {
    fn input_system(&mut self) -> &mut InputSystem {
        &mut self.input_system
    }
    fn update(&mut self, window: &mut GameWindow, _delta: f64) {
        let input_actions = self.input_system.actions();
        if input_actions.get("exit").just_pressed() {
            window.close();
        }
        self.example.update(&mut self.gui);
        self.gui.update(input_actions);
    }
}

fn main() {
    run_game(GuiGame::new);
}
