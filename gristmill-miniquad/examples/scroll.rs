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

const EXAMPLE_TEXT: &'static str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Etiam maximus ac turpis eget feugiat. Vivamus nibh sem, bibendum in neque vel, dictum lacinia mauris. Curabitur consequat et neque eu auctor. Aenean non nisi gravida, scelerisque odio in, rutrum elit. Donec vestibulum sem ultricies nisl lobortis accumsan. Nam ac posuere elit. Praesent nec leo non enim posuere sodales ut quis tortor. In vitae nisl convallis nisi rhoncus fringilla.

Cras luctus sem neque, in semper magna blandit nec. Sed vel luctus neque. Phasellus et turpis dictum, aliquam mi sed, imperdiet odio. Vivamus et cursus dolor. Suspendisse ac ligula efficitur, rutrum felis in, eleifend massa. Proin semper vestibulum quam, vel pulvinar risus convallis non. Mauris sed felis massa.";

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
}

impl MyGame {
    fn gui_event(&mut self, event: WidgetEvent) {
        let button_index: usize = *event.payload.unwrap().downcast().unwrap();
        log::trace!("Clicked Button {}", button_index);
    }
}

impl Game for MyGame {
    fn init(mut context: Context, screen_size: Vec2) -> Self {
        let assets = GameAssets::load();
        let mut text_brush = TextBrush::new(assets.fonts);
        let renderer = Renderer2D::new(&mut context, Some(text_brush.glyph_texture_size()));
        let viewport = Rect::from_min_size(Pos2::ZERO, screen_size);

        let mut gui = Gui::new();
        gui.layout_widget(
            (),
            &WidgetRc::new({
                let mut container = Container::new(Direction::Horizontal, CrossAxis::Start, "root");
                container.add_widget(ScrollArea::new("scroll-area", Direction::Vertical, {
                    let mut content = Label::new("scroll-content", EXAMPLE_TEXT);
                    content.autosize(&mut text_brush, 0);
                    content
                }));
                container.add_widget(ScrollArea::new("scroll-area", Direction::Vertical, {
                    let mut content =
                        Container::new(Direction::Vertical, CrossAxis::Stretch, "container");
                    for index in 1_usize..=7 {
                        let mut button =
                            Button::new("button", "button", format!("Button {}", index));
                        button.set_event_payload(index);
                        content.add_widget(button);
                    }
                    content
                }));
                container
            }),
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
        WindowSetup::from_title("Scroll Example".to_string()),
        WindowConfig::default(),
    );
}
