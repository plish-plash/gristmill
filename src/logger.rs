use std::{
    fs::File,
    io::Write,
    panic::PanicHookInfo,
    path::Path,
    sync::{LazyLock, Mutex},
};

use console::style;
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

static LOG_FILE: Mutex<Option<File>> = Mutex::new(None);

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
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
        let mut log_file = LOG_FILE.lock().unwrap();
        if let Some(file) = log_file.as_mut() {
            let res = writeln!(
                file,
                "{:5} [{}] {}",
                record.level(),
                record.target(),
                record.args()
            );
            if res.is_err() {
                *log_file = None;
            }
        }
    }
    fn flush(&self) {}
}

fn panic_handler(panic_info: &PanicHookInfo) {
    let mut log_file = LOG_FILE.lock();
    if let Ok(Some(file)) = log_file.as_mut().map(|opt| opt.as_mut()) {
        let _ = writeln!(file, "{}", panic_info);
    }
    PROGRESS_BAR.finish();
    eprintln!(
        "{}",
        style("The application has panicked. See the log for details.")
            .red()
            .for_stderr()
    );
}

pub fn init_logging(log_file: Option<&Path>) {
    log::set_logger(&Logger).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
    if let Some(log_file) = log_file {
        match File::create(log_file) {
            Ok(file) => {
                *LOG_FILE.lock().unwrap() = Some(file);
                std::panic::set_hook(Box::new(panic_handler));
            }
            Err(error) => log::error!("{}", error),
        }
    }
}
