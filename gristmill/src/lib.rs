pub mod asset;
pub mod event;
mod game;
pub mod geom2d;
pub mod input;
mod object;
pub mod render;

pub use downcast_rs::*;
pub use game::*;
pub use glam as math;
pub use object::*;
pub use palette as color;
pub use pareen as tween;

pub type Color = color::LinSrgba;

pub fn init_logging() {
    use env_logger::*;
    let default_log_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };
    Builder::from_env(Env::default().default_filter_or(default_log_level))
        .try_init()
        .ok();
}
