use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use gristmill::asset::{self, Asset, AssetError, YamlAsset};
use miniquad::conf::{Conf, Icon};
use serde::{Deserialize, Serialize};

pub use miniquad::window::{order_quit, request_quit, screen_size};

pub struct WindowSetup<'a> {
    pub title: String,
    pub icon: Option<&'a Path>,
    pub resizable: bool,
    pub config: &'a Path,
}

impl<'a> WindowSetup<'a> {
    pub fn with_title(title: String) -> Self {
        WindowSetup {
            title,
            icon: None,
            resizable: true,
            config: Path::new("window.yaml"),
        }
    }
    pub fn with_title_and_icon(title: String, icon: &'a Path) -> Self {
        WindowSetup {
            title,
            icon: Some(icon),
            resizable: true,
            config: Path::new("window.yaml"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_count: Option<u32>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            fullscreen: false,
            sample_count: None,
        }
    }
}
impl YamlAsset for WindowConfig {}

struct WindowIcon(Vec<u8>);

impl From<WindowIcon> for Icon {
    fn from(value: WindowIcon) -> Self {
        const SMALL_SIZE: usize = 16 * 16 * 4;
        const MEDIUM_SIZE: usize = 32 * 32 * 4;
        const LARGE_SIZE: usize = 64 * 64 * 4;
        assert_eq!(
            value.0.len(),
            SMALL_SIZE + MEDIUM_SIZE + LARGE_SIZE,
            "WindowIcon invalid data"
        );
        Icon {
            small: value.0[0..SMALL_SIZE].try_into().unwrap(),
            medium: value.0[SMALL_SIZE..(SMALL_SIZE + MEDIUM_SIZE)]
                .try_into()
                .unwrap(),
            big: value.0[(SMALL_SIZE + MEDIUM_SIZE)..].try_into().unwrap(),
        }
    }
}

static GLOBAL_CONFIG: Mutex<Option<(PathBuf, WindowConfig)>> = Mutex::new(None);

pub(crate) fn load_config(window_setup: WindowSetup, default_config: WindowConfig) -> Conf {
    let icon = window_setup.icon.and_then(|path| {
        asset::load_file(path)
            .and_then(|mut reader| {
                use std::io::Read;
                let mut icon = WindowIcon(Vec::new());
                reader
                    .read_to_end(&mut icon.0)
                    .map_err(|e| AssetError::new_io(path.to_owned(), false, e))?;
                Ok(icon)
            })
            .inspect_err(|e| log::error!("{}", e))
            .ok()
    });
    let config = WindowConfig::load(window_setup.config)
        .inspect_err(|e| log::warn!("{}", e))
        .unwrap_or(default_config);
    *GLOBAL_CONFIG.lock().unwrap() = Some((window_setup.config.to_owned(), config.clone()));
    Conf {
        window_title: window_setup.title,
        window_width: config.width as i32,
        window_height: config.height as i32,
        fullscreen: config.fullscreen,
        sample_count: config.sample_count.unwrap_or(1) as i32,
        window_resizable: window_setup.resizable,
        icon: icon.map(Into::into),
        ..Default::default()
    }
}
pub(crate) fn save_config() {
    let config = GLOBAL_CONFIG.lock().unwrap();
    if let Some((path, config)) = config.as_ref() {
        if let Err(error) = asset::save_yaml_file(path, config) {
            log::error!("{}", error);
        }
    }
}
pub(crate) fn on_resize(width: f32, height: f32) {
    let mut config = GLOBAL_CONFIG.lock().unwrap();
    if let Some((_, config)) = config.as_mut() {
        if !config.fullscreen {
            config.width = width as u32;
            config.height = height as u32;
        }
    }
}

pub fn get_fullscreen() -> bool {
    let config = GLOBAL_CONFIG.lock().unwrap();
    config
        .as_ref()
        .map(|(_, config)| config.fullscreen)
        .unwrap_or_default()
}
pub fn set_fullscreen(fullscreen: bool) {
    let mut config = GLOBAL_CONFIG.lock().unwrap();
    if let Some((_, config)) = config.as_mut() {
        config.fullscreen = fullscreen;
        miniquad::window::set_fullscreen(fullscreen);
        if !fullscreen {
            miniquad::window::set_window_size(config.width, config.height);
        }
    }
}
pub fn toggle_fullscreen() {
    let mut config = GLOBAL_CONFIG.lock().unwrap();
    if let Some((_, config)) = config.as_mut() {
        config.fullscreen = !config.fullscreen;
        miniquad::window::set_fullscreen(config.fullscreen);
        if !config.fullscreen {
            miniquad::window::set_window_size(config.width, config.height);
        }
    }
}
