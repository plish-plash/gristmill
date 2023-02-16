use gristmill::{
    geom2d::EdgeRect,
    gui::{
        widget::{Button, Text, WidgetNodeExt, WidgetStyles},
        Anchor, Gui, GuiNode, GuiNodeExt,
    },
    input::InputSystem,
    render::{RenderContext, Renderable},
    run_game, Game, GameWindow,
};

struct ButtonExample {
    button: Button,
    text: Text,
    times_clicked: usize,
}

impl ButtonExample {
    fn new(gui: &mut Gui) -> ButtonExample {
        let container = gui.root().add_child(gui, GuiNode::default());
        container.set_layout_margin(gui, EdgeRect::splat(64));
        container.set_layout_height(gui, 32);
        container.set_child_layout(gui, "hbox");
        container.set_child_spacing(gui, 8);

        let button: Button = gui.create_widget(container);
        button.set_layout_width(gui, 128);
        button.set_label_string(gui, "Click Me!");

        let text: Text = gui.create_widget(container);
        text.set_layout_width(gui, 128);
        text.set_text_align(gui, (Anchor::Begin, Anchor::Middle), false);

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
        let mut gui = Gui::new(context, WidgetStyles::default());
        let example = ButtonExample::new(&mut gui);
        GuiGame {
            input_system: InputSystem::load_config(),
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
