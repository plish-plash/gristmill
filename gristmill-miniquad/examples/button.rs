use std::path::Path;

use gristmill::{
    asset::Asset,
    color::Color,
    gui::*,
    math::{Rect, Vec2},
    scene2d::{sprite::ColorRect, ViewportCamera},
    style::StyleSheet,
    text::{FontAsset, Text, TextBrush},
    DrawMetrics,
};
use gristmill_miniquad::{Context, Game, InputEvent, MouseButton, Renderer2D, Scene2D};

type Layer = GuiSubLayer;

struct GameAssets {
    fonts: Vec<FontAsset>,
}

impl GameAssets {
    fn load() -> Self {
        StyleSheet::default()
            .load_global(Path::new("examples/assets/style.yaml"))
            .unwrap();
        GameAssets {
            fonts: Vec::<FontAsset>::load(Path::new("examples/assets/fonts.yaml")).unwrap(),
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

enum GuiPrimitive {
    ColorRect(ColorRect<Layer>),
    Text(Text<'static, Layer>),
}

impl GuiPrimitive {
    fn draw(&self, scene: &mut Scene2D<Layer>, text_brush: &mut TextBrush<Layer>) {
        match self {
            GuiPrimitive::ColorRect(color_rect) => color_rect.draw(scene),
            GuiPrimitive::Text(text) => text_brush.queue(text),
        }
    }
}
impl DrawPrimitive for GuiPrimitive {
    fn from_text(text: Text<'static, Layer>) -> Self {
        GuiPrimitive::Text(text)
    }
    fn from_button_background(rect: Rect, state: ButtonState) -> Self {
        let color = match state {
            ButtonState::Normal => Color::new_rgb(0.5, 0.5, 0.5),
            ButtonState::Hover => Color::new_rgb(0.6, 0.6, 0.6),
            ButtonState::Press => Color::new_rgb(0.4, 0.4, 0.4),
            ButtonState::Disable => Color::new_rgba(0.5, 0.5, 0.5, 0.5),
        };
        GuiPrimitive::ColorRect(ColorRect(Layer::Background, color, rect))
    }
}

struct MyGame {
    context: Context,
    renderer: Renderer2D,
    camera: ViewportCamera,
    scene: Scene2D<Layer>,
    text_brush: TextBrush<Layer>,
    gui_input: GuiInput,
    gui: Gui<(), GuiPrimitive>,
    container: Container<GuiPrimitive>,
    label: WidgetRef<Label<GuiPrimitive>>,
    times_clicked: u32,
}

impl MyGame {
    fn gui_event(&mut self, event: WidgetEvent) {
        if event.name == "button" {
            self.times_clicked += 1;
            self.label
                .borrow_mut()
                .set_text(format!("Times clicked: {}", self.times_clicked));
        }
    }
}

impl Game for MyGame {
    fn init(mut context: Context, screen_size: Vec2) -> Self {
        let assets = GameAssets::load();
        let text_brush = TextBrush::new(assets.fonts);
        let renderer = Renderer2D::new(&mut context, Some(text_brush.glyph_texture_size()));
        let camera = ViewportCamera { screen_size };

        let mut container = Container::new(Direction::Horizontal, CrossAxis::Start, "root");
        container.add_widget(Button::new("button", "button", "Click Me"));
        let label = container.add_widget(Label::new("label", ""));
        let mut gui = Gui::new();
        gui.layout((), &container, camera.viewport());

        MyGame {
            context,
            renderer,
            camera,
            scene: Scene2D::new(),
            text_brush,
            gui_input: GuiInput::new(),
            gui,
            container,
            label,
            times_clicked: 0,
        }
    }

    fn input(&mut self, event: InputEvent) {
        if let Some(event) = GuiInputEvent::from_input(&event, gui_mouse_button) {
            self.gui_input.process(event);
        }
    }

    fn update(&mut self, _dt: f32) {
        if let Some(event) = self.gui.handle_input(&mut self.gui_input) {
            self.gui_event(event);
        }
    }

    fn resize(&mut self, screen_size: Vec2) {
        self.camera.screen_size = screen_size;
        self.gui.layout((), &self.container, self.camera.viewport());
    }

    fn draw(&mut self) -> DrawMetrics {
        for primitive in self.gui.draw(&()) {
            primitive.draw(&mut self.scene, &mut self.text_brush);
        }
        self.text_brush.draw(
            &mut self.context,
            self.renderer.glyph_texture(),
            &mut self.scene,
        );

        self.renderer.begin_render(&mut self.context, Color::BLACK);
        self.renderer
            .set_camera(&mut self.context, self.camera.transform());
        self.scene.draw(&mut self.context, &mut self.renderer, ..);
        self.renderer.end_render(&mut self.context)
    }
}

fn main() {
    gristmill_miniquad::start::<MyGame>(Default::default());
}
