use gristmill::{
    asset::Resources,
    geom2d::Rect,
    input::{ActionState, CursorAction, InputActions, InputSystem},
    math::IVec2,
    render::RenderContext,
    run_game, Color, Game, Window,
};
use gristmill_gui::{
    widget::{Button, ButtonStyle, Text, TextAlign, Widget},
    Gui, GuiInputActions, GuiLayout, GuiRenderer,
};

// -------------------------------------------------------------------------------------------------

struct Scene {
    gui: Gui,
}

#[derive(Default)]
struct GuiGameInput {
    primary: CursorAction,
}

impl InputActions for GuiGameInput {
    fn end_frame(&mut self) {
        self.primary.end_frame();
    }
    fn set_action_state(&mut self, target: &str, state: ActionState) {
        if target == "primary" {
            self.primary.set_state(state);
        }
    }
}

impl GuiInputActions for GuiGameInput {
    fn primary(&self) -> &CursorAction {
        &self.primary
    }
}

struct ButtonExample {
    button: Button,
    text: Text,
    times_clicked: usize,
}

impl ButtonExample {
    fn new(gui: &mut Gui) -> Self {
        let root = gui.root();
        let button_size = ButtonStyle::default().size;

        let button = Button::create(gui, root.clone(), None);
        button.set_layout(GuiLayout::Child(Rect::new(IVec2::new(32, 32), button_size)));
        button.set_label_string("Click Me!");

        let mut text = Text::create(gui, root, None);
        text.set_layout(GuiLayout::Child(Rect::new(
            IVec2::new(32 + 128 + 8, 32),
            button_size,
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
    input: GuiGameInput,
    example: ButtonExample,
    gui_renderer: GuiRenderer,
}

impl Game for GuiGame {
    fn load(_resources: Resources, context: &mut RenderContext) -> Self {
        let mut gui = Gui::new();
        let example = ButtonExample::new(&mut gui);

        GuiGame {
            scene: Scene { gui },
            input: Default::default(),
            example,
            gui_renderer: GuiRenderer::new(context),
        }
    }

    fn update(&mut self, _window: &Window, input_system: &mut InputSystem, _delta: f64) {
        input_system.dispatch_queue(&mut self.input);
        self.example.update();
        self.scene.gui.update(&self.input);
    }

    fn render(&mut self, context: &mut RenderContext) {
        self.gui_renderer.pre_render(context, &mut self.scene.gui);
        context.begin_render_pass(Color::new(0.0, 0.5, 0.5, 1.0));
        self.gui_renderer.render(context);
        context.end_render_pass();
    }
}

fn main() {
    run_game::<GuiGame>(Resources::new());
}
