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

const EXAMPLE_TEXT: &'static str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Etiam maximus ac turpis eget feugiat. Vivamus nibh sem, bibendum in neque vel, dictum lacinia mauris. Curabitur consequat et neque eu auctor. Aenean non nisi gravida, scelerisque odio in, rutrum elit. Donec vestibulum sem ultricies nisl lobortis accumsan. Nam ac posuere elit. Praesent nec leo non enim posuere sodales ut quis tortor. In vitae nisl convallis nisi rhoncus fringilla.

Cras luctus sem neque, in semper magna blandit nec. Sed vel luctus neque. Phasellus et turpis dictum, aliquam mi sed, imperdiet odio. Vivamus et cursus dolor. Suspendisse ac ligula efficitur, rutrum felis in, eleifend massa. Proin semper vestibulum quam, vel pulvinar risus convallis non. Mauris sed felis massa.";

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
}

impl Game {
    fn gui_event(&mut self, event: WidgetEvent) {
        let button_index: usize = *event.payload.unwrap().downcast().unwrap();
        log::trace!("Clicked Button {}", button_index);
    }
}

impl gristmill_miniquad::Game for Game {
    fn init(context: Context, screen_size: Vec2) -> Self {
        let assets = GameAssets::load();
        let mut renderer = GameRenderer::new(context, assets.fonts);
        let viewport = Rect::from_min_size(Pos2::ZERO, screen_size);

        let mut gui = Gui::new();
        gui.layout(
            (),
            {
                let mut container = Container::new(Direction::Horizontal, CrossAxis::Start, "root");
                container.add(ScrollArea::new("scroll-area", Direction::Vertical, {
                    let mut content = Label::new("scroll-content", EXAMPLE_TEXT);
                    content.autosize(&mut renderer.text_brush, 0);
                    content
                }));
                container.add(ScrollArea::new("scroll-area", Direction::Vertical, {
                    let mut content =
                        Container::new(Direction::Vertical, CrossAxis::Stretch, "container");
                    for index in 1_usize..=7 {
                        let mut button =
                            Button::new("button", "button", format!("Button {}", index));
                        button.set_event_payload(index);
                        content.add(button);
                    }
                    content
                }));
                container
            },
            viewport,
        );

        Game { renderer, gui }
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
        WindowSetup::from_title("Scroll Example".to_string()),
        WindowConfig::default(),
    );
}
