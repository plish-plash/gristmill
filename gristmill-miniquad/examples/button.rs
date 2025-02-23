use std::path::Path;

use gristmill::{
    asset::Asset,
    color::Color,
    gui::*,
    math::{Pos2, Rect, Vec2},
    scene2d::sprite::ColorRect,
    style::StyleSheet,
    text::{FontAsset, Text, TextBrush},
    DrawMetrics,
};
use gristmill_miniquad::{
    Context, DrawParams, Game, InputEvent, MouseButton, Renderer2D, WindowConfig, WindowSetup,
};

type Layer = usize;

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

enum GuiPrimitive {
    ColorRect(ColorRect),
    Text(Text<'static>),
}

impl Primitive for GuiPrimitive {
    type Layer = Layer;
    type Params = DrawParams;
    fn layer(index: usize) -> Layer {
        index
    }
    fn from_text(text: Text<'static>) -> Self {
        GuiPrimitive::Text(text)
    }
    fn from_button(rect: Rect, state: ButtonState) -> Self {
        let color = match state {
            ButtonState::Normal => Color::new_rgb(0.5, 0.5, 0.5),
            ButtonState::Hover => Color::new_rgb(0.6, 0.6, 0.6),
            ButtonState::Press => Color::new_rgb(0.4, 0.4, 0.4),
            ButtonState::Disable => Color::new_rgba(0.5, 0.5, 0.5, 0.5),
        };
        GuiPrimitive::ColorRect(ColorRect(color, rect))
    }
    fn draw(self, stage: &mut GuiStage<Self>, text_brush: &mut TextBrush<Layer>, layer: Layer) {
        match self {
            GuiPrimitive::ColorRect(color_rect) => {
                color_rect.draw(stage.get_layer(layer), DrawParams::new_fill(-1))
            }
            GuiPrimitive::Text(text) => text_brush.queue(layer, &text),
        }
    }
}

struct GuiRenderer {
    context: Context,
    renderer: Renderer2D,
    stage: GuiStage<GuiPrimitive>,
    text_brush: TextBrush<Layer>,
    viewport: Rect,
}

impl GuiRenderer {
    fn render(&mut self, gui: &mut Gui<(), GuiPrimitive>) -> DrawMetrics {
        gui.draw(&mut self.stage, &mut self.text_brush, self.viewport);
        self.text_brush.draw(
            &mut self.context,
            self.renderer.glyph_texture(),
            &mut self.stage,
        );
        self.renderer
            .render(&mut self.context, &mut self.stage, Color::BLACK)
    }
}

struct MyGame {
    renderer: GuiRenderer,
    gui: Gui<(), GuiPrimitive>,
    label: WidgetRc<Label<GuiPrimitive>>,
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

        MyGame {
            renderer: GuiRenderer {
                context,
                renderer,
                stage: GuiStage::<GuiPrimitive>::new(),
                text_brush,
                viewport,
            },
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
        self.renderer.viewport = Rect::from_min_size(Pos2::ZERO, screen_size);
        self.gui.relayout(self.renderer.viewport);
    }

    fn draw(&mut self) -> DrawMetrics {
        self.renderer.render(&mut self.gui)
    }
}

fn main() {
    gristmill::asset::set_base_path("examples/assets").unwrap();
    gristmill_miniquad::start::<MyGame>(
        WindowSetup::from_title("Button Example".to_string()),
        WindowConfig::default(),
    );
}
