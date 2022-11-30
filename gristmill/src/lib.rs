pub mod asset;
mod game;
pub mod geom2d;
pub mod input;
pub mod object;
pub mod render;

pub use game::*;
pub use glam as math;
pub use palette as color;
pub use pareen as tween;

use log::*;
use std::sync::mpsc;

pub type Color = color::LinSrgba;

pub struct LogRecord {
    pub level: Level,
    pub target: String,
    pub message: String,
}

struct CustomLogger(env_logger::Logger, mpsc::SyncSender<LogRecord>);

impl Log for CustomLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }
    fn log(&self, record: &Record) {
        self.0.log(record);
        self.1
            .try_send(LogRecord {
                level: record.level(),
                target: record.target().to_owned(),
                message: format!("{}", record.args()),
            })
            .ok();
    }
    fn flush(&self) {
        self.0.flush();
    }
}

fn env_logger_builder() -> env_logger::Builder {
    let default_log_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_log_level))
}
pub fn init_logging() {
    env_logger_builder().try_init().ok();
}
pub fn init_custom_logging() -> mpsc::Receiver<LogRecord> {
    let logger = env_logger_builder().build();
    let log_level = logger.filter();
    let (sender, receiver) = mpsc::sync_channel(100);
    set_boxed_logger(Box::new(CustomLogger(logger, sender))).ok();
    set_max_level(log_level);
    receiver
}
