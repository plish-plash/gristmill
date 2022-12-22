use crate::{
    gui::{
        widget::{Text, TextAlign, WidgetNode, WidgetStyles},
        Gui, GuiLayout, OwnedText,
    },
    input::InputSystem,
    render::{RenderContext, Renderable},
    Game, GameWindow,
};
use log::Level;
use std::{collections::VecDeque, sync::mpsc};

pub struct LogRecord {
    pub level: Level,
    pub target: String,
    pub message: String,
}

fn log_level_color(level: Level) -> [f32; 4] {
    match level {
        Level::Error => [1.0, 0.0, 0.0, 1.0],
        Level::Warn => [1.0, 1.0, 0.0, 1.0],
        Level::Info => [0.0, 1.0, 0.0, 1.0],
        Level::Debug => [0.0, 0.0, 1.0, 1.0],
        Level::Trace => [0.0, 1.0, 1.0, 1.0],
    }
}

pub struct ConsoleGame<G: Game> {
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
        self.text.set_text(&mut self.gui, text);
    }

    pub fn new(
        context: &mut RenderContext,
        log_receiver: mpsc::Receiver<LogRecord>,
        game: G,
    ) -> Self {
        let mut gui = Gui::with_styles(context, WidgetStyles::default());
        let text: Text = gui.create_widget(gui.root(), None);
        text.set_layout(&mut gui, GuiLayout::fill());
        text.set_align(&mut gui, TextAlign::LeftWrap);
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

impl<G: Game> Renderable for ConsoleGame<G> {
    fn pre_render(&mut self, context: &mut RenderContext) {
        if self.show_console {
            self.gui.pre_render(context);
        } else {
            self.game.pre_render(context);
        }
    }
    fn render(&mut self, context: &mut RenderContext) {
        if self.show_console {
            self.gui.render(context);
        } else {
            self.game.render(context);
        }
    }
}

impl<G: Game> Game for ConsoleGame<G> {
    fn input_system(&mut self) -> &mut InputSystem {
        self.game.input_system()
    }
    fn update(&mut self, window: &mut GameWindow, delta: f64) {
        if self.input_system().actions().get("console").just_pressed() {
            self.show_console = !self.show_console;
        }
        while let Ok(record) = self.log_receiver.try_recv() {
            if record.level == Level::Error {
                self.show_console = true;
            }
            self.push_record(record);
        }

        if self.show_console {
            let input = self.game.input_system().actions();
            self.gui.update(input);
            if input
                .try_get("exit")
                .map(|a| a.just_pressed())
                .unwrap_or(false)
            {
                window.close();
            }
        } else {
            self.game.update(window, delta);
        }
    }
}
