use crate::{
    widget::{Text, TextAlign},
    Gui, GuiLayout, GuiRenderer, Widget,
};
use glyph_brush::OwnedText;
use gristmill::{
    geom2d::Size, input::InputActions, render::RenderContext, Color, Game, GameWindow, LogRecord,
};
use log::Level;
use std::{collections::VecDeque, sync::mpsc};

fn log_level_color(level: Level) -> [f32; 4] {
    match level {
        Level::Error => [1.0, 0.0, 0.0, 1.0],
        Level::Warn => [1.0, 1.0, 0.0, 1.0],
        Level::Info => [0.0, 1.0, 0.0, 1.0],
        Level::Debug => [0.0, 0.0, 1.0, 1.0],
        Level::Trace => [0.0, 1.0, 1.0, 1.0],
    }
}

struct ConsoleGame<G: Game> {
    log_receiver: mpsc::Receiver<LogRecord>,
    buffer: VecDeque<LogRecord>,
    gui: Gui,
    text: Text,
    show_console: bool,
    game: G,
}

impl<G: Game> ConsoleGame<G> {
    fn record_to_text(record: &LogRecord, text: &mut Vec<OwnedText>) {
        const TARGET_COLOR: [f32; 4] = [0.4, 0.4, 0.4, 1.0];
        text.push(OwnedText::new("["));
        text.push(
            OwnedText::new(format!("{} ", record.level)).with_color(log_level_color(record.level)),
        );
        text.push(OwnedText::new(record.target.to_string()).with_color(TARGET_COLOR));
        text.push(OwnedText::new(format!("] {}\n", record.message)));
    }
    fn push_record(&mut self, record: LogRecord) {
        self.buffer.push_back(record);
        // TODO scrollback
        while self.buffer.len() > 20 {
            self.buffer.pop_front();
        }
        let mut text = Vec::new();
        for record in self.buffer.iter() {
            Self::record_to_text(record, &mut text);
        }
        self.text.set_text(text);
    }

    fn new(log_receiver: mpsc::Receiver<LogRecord>, game: G) -> Self {
        let mut gui = Gui::default();
        let text: Text = gui.create_widget(gui.root(), None);
        text.set_layout(GuiLayout::fill());
        text.set_align(TextAlign::LeftWrap);
        ConsoleGame {
            log_receiver,
            buffer: VecDeque::new(),
            gui,
            text,
            show_console: false,
            game,
        }
    }
}

impl<G: Game> Game for ConsoleGame<G> {
    type Renderer = (GuiRenderer, G::Renderer);
    fn resize(&mut self, dimensions: Size) {
        self.game.resize(dimensions);
    }
    fn update(&mut self, window: &mut GameWindow, input: &InputActions, delta: f64) -> Option<()> {
        if input
            .get("console")
            .map(|a| a.just_pressed())
            .unwrap_or(false)
        {
            self.show_console = !self.show_console;
        }
        while let Ok(record) = self.log_receiver.try_recv() {
            if record.level == Level::Error {
                self.show_console = true;
            }
            self.push_record(record);
        }

        if self.show_console {
            self.gui.update(input);
            if input
                .try_get("exit")
                .map(|a| a.just_pressed())
                .unwrap_or(false)
            {
                window.close();
            }
        } else {
            self.game.update(window, input, delta);
        }
        Some(())
    }
    fn render(
        &mut self,
        context: &mut RenderContext,
        (gui_renderer, game_renderer): &mut Self::Renderer,
    ) {
        if self.show_console {
            gui_renderer.process(context, &mut self.gui);
            context.begin_render_pass(Color::new(0.9, 0.9, 0.9, 1.0));
            gui_renderer.draw_all(context);
            context.end_render_pass();
        } else {
            self.game.render(context, game_renderer);
        }
    }
}

pub fn run_game_with_console<G, F>(func: F) -> !
where
    G: Game,
    F: FnOnce() -> G,
{
    let log_receiver = gristmill::init_custom_logging();
    gristmill::run_game(|| ConsoleGame::new(log_receiver, func()))
}
