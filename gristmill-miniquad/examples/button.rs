use std::path::Path;

use gristmill::{
    asset::Asset,
    color::Color,
    gui::*,
    math::{Pos2, Rect, Vec2},
    style::StyleSheet,
    text::{FontAsset, TextBrush},
    DrawMetrics,
};
use gristmill_miniquad::{
    Context, InputEvent, Material, MouseButton, Pipeline2D, Renderer2D, WindowConfig, WindowSetup,
};

struct GameAssets {
    fonts: Vec<FontAsset>,
}

impl GameAssets {
    fn load() -> Self {
        StyleSheet::default()
            .load_global(Path::new("style.yaml"))
            .unwrap();
        GameAssets {
            fonts: Vec::<FontAsset>::load(Path::new("fonts.yaml")).unwrap(),
        }
    }
}

fn gui_mouse_button(button: MouseButton) -> Option<GuiMouseButton> {
    match button {
        MouseButton::Left => Some(GuiMouseButton::Primary),
        MouseButton::Right => Some(GuiMouseButton::Secondary),
        _ => None,
    }
}

struct GuiRenderer;

impl gristmill::gui::GuiRenderer for GuiRenderer {
    type GuiLayer = ();
    type TextLayer = usize;
    type Pipeline = Pipeline2D;
    fn text_layer(_layer: &(), sublayer: usize) -> usize {
        sublayer
    }
    fn button_material(&self, _state: ButtonState) -> Material {
        Material::SOLID
    }
}

struct GameRenderer {
    renderer: Renderer2D,
    text_brush: TextBrush<Pipeline2D, usize>,
}

impl GameRenderer {
    fn new(context: Context, fonts: Vec<FontAsset>) -> Self {
        let text_brush = TextBrush::new(fonts);
        let renderer = Renderer2D::new_text(context, &text_brush);
        GameRenderer {
            renderer,
            text_brush,
        }
    }
}

struct Game {
    renderer: GameRenderer,
    gui: Gui<GuiRenderer>,
    label: WidgetRc<Label<GuiRenderer>>,
    times_clicked: u32,
}

impl Game {
    fn gui_event(&mut self, event: WidgetEvent) {
        if event.name == "button" {
            self.times_clicked += 1;
            self.label
                .borrow_mut()
                .set_text(format!("Times clicked: {}", self.times_clicked));
        }
    }
}

impl gristmill_miniquad::Game for Game {
    fn init(context: Context, screen_size: Vec2) -> Self {
        let assets = GameAssets::load();
        let renderer = GameRenderer::new(context, assets.fonts);
        let viewport = Rect::from_min_size(Pos2::ZERO, screen_size);

        let label;
        let mut gui = Gui::new();
        gui.layout(
            (),
            {
                let mut container = Container::new(Direction::Horizontal, CrossAxis::Start, "root");
                container.add(Button::new("button", "button", "Click Me"));
                label = container.add(Label::new("label", ""));
                container
            },
            viewport,
        );

        Game {
            renderer,
            gui,
            label,
            times_clicked: 0,
        }
    }

    fn input(&mut self, event: InputEvent) {
        self.gui
            .process_input(GuiInputEvent::from_input(&event, gui_mouse_button));
    }

    fn update(&mut self, _dt: f32) {
        if let WidgetInput::Event(event) = self.gui.update_input() {
            self.gui_event(event);
        }
    }

    fn resize(&mut self, screen_size: Vec2) {
        self.gui
            .relayout(Rect::from_min_size(Pos2::ZERO, screen_size));
    }

    fn draw(&mut self) -> DrawMetrics {
        self.gui
            .draw_text(&GuiRenderer, &mut self.renderer.text_brush);
        self.renderer
            .renderer
            .process_text(&mut self.renderer.text_brush);
        self.renderer.renderer.begin_render(Color::BLACK);
        let mut batcher = self.renderer.renderer.bind_pipeline();
        self.gui
            .draw(&GuiRenderer, &mut self.renderer.text_brush, &mut batcher);
        std::mem::drop(batcher);
        self.renderer.renderer.end_render()
    }
}

fn main() {
    gristmill::asset::set_base_path("examples/assets").unwrap();
    gristmill_miniquad::start::<Game>(
        WindowSetup::from_title("Button Example".to_string()),
        WindowConfig::default(),
    );
}
