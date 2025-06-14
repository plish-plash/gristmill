mod error;
mod image;
pub mod locale;
pub mod particles;
pub mod util;
mod window;
pub mod world2d;

use std::{
    fs::{File, OpenOptions},
    io::Write,
    panic::PanicHookInfo,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, OnceLock,
    },
};

pub use euclid as math;
pub use silica_gui as gui;
pub use silica_gui::Rgba;
use silica_gui::{
    glyphon::{fontdb, FontSystem},
    theme::StandardThemeLoader,
};
pub use silica_wgpu as render;
use silica_wgpu::{wgpu, Context, SurfaceSize};
pub use winit::{event as input, keyboard};

pub use crate::{
    error::{GameError, ResultExt},
    image::*,
    window::*,
};

pub struct LocalSpace;
pub struct WorldSpace;
pub struct ScreenSpace;

#[derive(Debug)]
pub struct GameInfo {
    pub package_name: &'static str,
    pub package_version: &'static str,
    pub window_title: &'static str,
}

#[macro_export]
macro_rules! game_info {
    ($window_title:expr) => {
        $crate::GameInfo {
            package_name: env!("CARGO_PKG_NAME"),
            package_version: env!("CARGO_PKG_VERSION"),
            window_title: $window_title,
        }
    };
}

static GAME_INFO: OnceLock<GameInfo> = OnceLock::new();
static DEFAULT_PANIC_HOOK: OnceLock<Box<dyn Fn(&PanicHookInfo<'_>) + Send + Sync>> =
    OnceLock::new();
static HAS_PANICKED: AtomicBool = AtomicBool::new(false);

pub trait Game: Sized {
    fn load(context: &Context, surface_format: wgpu::TextureFormat) -> Result<Self, GameError>;
    fn resize(&mut self, context: &Context, size: SurfaceSize);
    fn input_event(&mut self, event: InputEvent);
    fn update(&mut self, dt: f32);
    fn clear_color(&self) -> Rgba;
    fn render(&mut self, context: &Context, pass: &mut wgpu::RenderPass);
}

fn panic_hook(panic_info: &PanicHookInfo) {
    const CRASH_LOG_FILE: &str = "CRASH.txt";
    if let Some(default_hook) = DEFAULT_PANIC_HOOK.get() {
        default_hook(panic_info);
    }
    if let Some(game_info) = GAME_INFO.get() {
        let result = (|| {
            if HAS_PANICKED.swap(true, Ordering::Relaxed) {
                let mut output = OpenOptions::new().append(true).open(CRASH_LOG_FILE)?;
                writeln!(output)?;
                writeln!(output, "{}", panic_info)
            } else {
                let mut output = File::create(CRASH_LOG_FILE)?;
                writeln!(
                    output,
                    "{} v{}",
                    game_info.package_name, game_info.package_version
                )?;
                writeln!(
                    output,
                    "Running on {} {}",
                    std::env::consts::OS,
                    std::env::consts::ARCH
                )?;
                writeln!(output)?;
                writeln!(output, "{}", panic_info)
            }
        })();
        match result {
            Ok(()) => eprintln!("panic message written to {}", CRASH_LOG_FILE),
            Err(error) => eprintln!("failed to write {}: {}", CRASH_LOG_FILE, error),
        }
    }
}

fn setup_environment(game_info: GameInfo) {
    env_logger::builder()
        .filter_level(if cfg!(debug_assertions) {
            log::LevelFilter::Trace
        } else {
            log::LevelFilter::Info
        })
        .filter_module("calloop", log::LevelFilter::Info)
        .filter_module("wgpu_core", log::LevelFilter::Info)
        .filter_module("wgpu_hal", log::LevelFilter::Warn)
        .filter_module("naga", log::LevelFilter::Info)
        .filter_module("cosmic_text", log::LevelFilter::Info)
        .parse_default_env()
        .init();

    #[cfg(debug_assertions)]
    {
        std::env::set_current_dir("assets").expect("failed to set current directory");
    }
    #[cfg(not(debug_assertions))]
    {
        let mut exe_dir =
            std::env::current_exe().expect("could not get path of current executable");
        exe_dir.pop();
        std::env::set_current_dir(&exe_dir).expect("failed to set current directory");
    }

    log::info!("{} v{}", game_info.package_name, game_info.package_version);
    log::info!(
        "Powered by {} v{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    log::info!(
        "Running on {} {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    let _ = GAME_INFO.set(game_info);
    let _ = DEFAULT_PANIC_HOOK.set(std::panic::take_hook());
    std::panic::set_hook(Box::new(panic_hook));
}

pub fn load_asset<P, T, F>(path: P, f: F) -> Result<T, GameError>
where
    P: AsRef<Path>,
    F: FnOnce(&Path) -> Result<T, GameError>,
{
    let path = path.as_ref();
    log::debug!("Loading {}", path.display());
    f(path).map_err(|e| e.with_read(path.to_path_buf()))
}
pub fn save_asset<P, T, F>(path: P, f: F) -> Result<T, GameError>
where
    P: AsRef<Path>,
    F: FnOnce(&Path) -> Result<T, GameError>,
{
    let path = path.as_ref();
    log::debug!("Saving {}", path.display());
    f(path).map_err(|e| e.with_write(path.to_path_buf()))
}

pub fn read_directory<P, F>(path: P, mut f: F) -> Result<(), GameError>
where
    P: AsRef<Path>,
    F: FnMut(&Path) -> Result<(), GameError>,
{
    let path_buf = path.as_ref().to_path_buf();
    let mut entries: Vec<_> = std::fs::read_dir(path)
        .map_err(|e| GameError::from_string(e.to_string()).with_read(path_buf.clone()))?
        .filter_map(|res| {
            res.ok()
                .filter(|e| e.file_type().unwrap().is_file())
                .map(|e| e.path())
        })
        .collect();
    entries.sort();
    log::info!(
        "Loading {} files from {}",
        entries.len(),
        path_buf.display()
    );
    for path in entries {
        f(&path)?;
    }
    Ok(())
}

pub fn get_locale() -> String {
    sys_locale::get_locale().unwrap_or_else(|| {
        log::warn!("failed to get system locale, falling back to en-US");
        "en-US".to_string()
    })
}

pub fn load_fonts() -> Result<FontSystem, GameError> {
    let mut db = fontdb::Database::new();
    read_directory("fonts", |path| {
        db.load_font_source(load_asset(path, |path| {
            Ok(fontdb::Source::Binary(Arc::new(std::fs::read(path)?)))
        })?);
        Ok(())
    })?;
    Ok(FontSystem::new_with_locale_and_db(get_locale(), db))
}

pub fn load_gui_theme() -> Result<StandardThemeLoader<'static>, GameError> {
    let image = image::Image::load("theme.png")?;
    Ok(StandardThemeLoader::new(image.data))
}
