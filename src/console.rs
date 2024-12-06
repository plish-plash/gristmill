use std::sync::LazyLock;

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use log::Level;

static PROGRESS_BAR: LazyLock<ProgressBar> = LazyLock::new(|| {
    let pb = ProgressBar::with_draw_target(None, ProgressDrawTarget::stderr_with_hz(2));
    pb.set_style(ProgressStyle::with_template("[{elapsed_precise}] {msg}").unwrap());
    pb
});

pub fn set_message(msg: impl Into<std::borrow::Cow<'static, str>>) {
    PROGRESS_BAR.set_message(msg);
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        use console::style;
        let level = match record.level() {
            Level::Error => style("ERROR").bold().red(),
            Level::Warn => style("WARN ").bold().yellow(),
            Level::Info => style("INFO ").bold(),
            Level::Debug => style("DEBUG").bold().blue(),
            Level::Trace => style("TRACE").bold().dim(),
        };
        let target = style(format!("[{}]", record.target())).dim();
        PROGRESS_BAR.suspend(|| {
            println!("{} {} {}", level, target, record.args());
        });
    }
    fn flush(&self) {}
}

pub fn init_logging() {
    log::set_logger(&Logger).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
}
