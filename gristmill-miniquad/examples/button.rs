use std::{path::Path, time::Duration};

use emath::Rect;
use gristmill::{
    asset::Asset, color::default_gui_palette, gui::*, render2d::Camera, text::FontAsset,
};
use gristmill_miniquad::{GameRenderer, InputEvent, WindowConfig};

struct GameAssets;

impl gristmill_miniquad::GameAssets for GameAssets {
    type GameState = GameState;
    fn window_config(&self) -> WindowConfig {
        WindowConfig::default()
    }
    fn fonts(&self) -> Vec<FontAsset> {
        Vec::<FontAsset>::load(Path::new("examples/assets/fonts.yaml")).unwrap()
    }
}

#[derive(Default)]
struct GameInput;

impl gristmill_miniquad::GameInput for GameInput {
    fn event(&mut self, _event: InputEvent) {}
}

struct GameState {
    gui: Container,
    layout: ContainerLayout,
    label: WidgetRef<Label>,
    times_clicked: u32,
}

impl GameState {
    fn gui_event(&mut self, event: GuiEvent) {
        if event.name == "button" {
            self.times_clicked += 1;
            self.label.borrow_mut().text = format!("Times clicked: {}", self.times_clicked).into();
        }
    }
}

impl gristmill_miniquad::GameState for GameState {
    type Assets = GameAssets;
    type Input = GameInput;

    fn new(_assets: GameAssets) -> Self {
        let palette = default_gui_palette();
        let button: WidgetRef<_> = Button::new(
            "button",
            ButtonPalette::new(&palette),
            Label::new("Click Me"),
        )
        .into();
        let label: WidgetRef<_> = Label::new("").into();
        let gui = Container::with_items(
            Direction::Horizontal,
            CrossAxis::Start,
            Padding::all(8.0),
            vec![button.item(), label.item()],
        );
        GameState {
            gui,
            layout: ContainerLayout::new(),
            label,
            times_clicked: 0,
        }
    }

    fn update(&mut self, _input: &mut GameInput, _frame_time: Duration) {}

    fn camera(&self) -> Camera {
        Default::default()
    }

    fn draw(&mut self, renderer: &mut GameRenderer, viewport: Rect, gui_input: GuiInputFrame) {
        self.layout.layout(&self.gui, viewport);
        if let Some(event) = self.layout.handle_input(&gui_input) {
            self.gui_event(event);
        }
        self.layout.draw(renderer);
    }
}

fn main() {
    gristmill_miniquad::start(|| GameAssets);
}
