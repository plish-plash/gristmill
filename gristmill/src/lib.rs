pub mod asset;
pub mod color;
pub mod event;
pub mod game;
pub mod geometry2d;
pub mod input;
pub mod renderer;
pub mod util;

pub fn init_logging() {
    use env_logger::*;
    let default_log_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };
    Builder::from_env(Env::default().default_filter_or(default_log_level))
        .try_init().ok();
}