use std::{fmt::Display, path::PathBuf};

use silica_gui::{glyphon::cosmic_text::Align, *};
use silica_wgpu::{wgpu, Context, SurfaceSize, TextureConfig};

use crate::{Game, InputEvent};

#[derive(Debug)]
pub struct GameError {
    asset: Option<(PathBuf, bool)>,
    message: String,
}

impl GameError {
    pub fn from_string(message: String) -> Self {
        GameError {
            asset: None,
            message,
        }
    }
    pub fn with_read(self, asset: PathBuf) -> Self {
        GameError {
            asset: Some((asset, false)),
            ..self
        }
    }
    pub fn with_write(self, asset: PathBuf) -> Self {
        GameError {
            asset: Some((asset, true)),
            ..self
        }
    }
}
impl<T: std::error::Error> From<T> for GameError {
    fn from(value: T) -> Self {
        GameError {
            asset: None,
            message: value.to_string(),
        }
    }
}
impl Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some((asset, write)) = self.asset.as_ref() {
            write!(
                f,
                "Error {} {}: {}",
                if *write { "writing" } else { "reading" },
                asset.display(),
                self.message
            )
        } else {
            f.write_str(&self.message)
        }
    }
}

pub trait ResultExt<T> {
    #[track_caller]
    fn unwrap_display(self) -> T;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Display,
{
    #[track_caller]
    fn unwrap_display(self) -> T {
        match self {
            Ok(t) => t,
            Err(e) => panic!("{}", e),
        }
    }
}

pub(crate) fn io_data_error(unsupported: bool, message: String) -> std::io::Error {
    let kind = if unsupported {
        std::io::ErrorKind::Unsupported
    } else {
        std::io::ErrorKind::InvalidData
    };
    std::io::Error::new(kind, message)
}

pub(crate) enum LoadGame<T> {
    NotLoaded,
    Game(T),
    Error(Gui, GuiRenderer),
}

impl<T: Game> LoadGame<T> {
    fn load_error_renderer(context: &Context, surface_format: wgpu::TextureFormat) -> GuiRenderer {
        let theme = crate::load_gui_theme().unwrap_display();
        let texture_config = TextureConfig::new(context, wgpu::FilterMode::Linear);
        GuiRenderer::new(context, surface_format, &texture_config, theme)
    }
    pub fn load(&mut self, context: &Context, surface_format: wgpu::TextureFormat) {
        *self = match T::load(context, surface_format) {
            Ok(game) => LoadGame::Game(game),
            Err(error) => {
                let error = error.to_string();
                log::error!("{}", error);
                let mut gui = Gui::new(crate::load_fonts().unwrap_display());
                {
                    let gui = &mut gui;
                    let dialog_layout = layout!(
                        border: Rect::length(1.0),
                        padding: Rect { top: length(16.0), ..Rect::length(8.0) },
                        gap: Size::length(16.0),
                        align_items: Some(AlignItems::Center),
                        flex_direction: FlexDirection::Column,
                    );
                    let root = gui! {
                        Node(layout: layout!(align_items: Some(AlignItems::Center), justify_content: Some(JustifyContent::Center))) {
                            Visible(layout: dialog_layout) {
                                Label(text: &error, font_size: 20.0, alignment: Some(Align::Center), layout: layout!(size: Size { width: Dimension::length(480.0), height: Dimension::auto() })),
                                Node(layout: layout!(gap: Size::length(8.0), align_items: Some(AlignItems::Center))) {
                                    Button(label: Some("Reload")) |_: &mut Gui| crate::reload(),
                                    Button(label: Some("Exit"), theme: ButtonTheme::Delete) |_: &mut Gui| crate::exit(),
                                }
                            }
                        }
                    };
                    gui.set_root(root);
                }
                LoadGame::Error(gui, Self::load_error_renderer(context, surface_format))
            }
        };
    }
    pub fn resize(&mut self, context: &Context, size: SurfaceSize) {
        match self {
            LoadGame::NotLoaded => (),
            LoadGame::Game(game) => game.resize(context, size),
            LoadGame::Error(gui, renderer) => {
                renderer.surface_resize(context, size);
                gui.set_available_space(taffy::Size {
                    width: taffy::AvailableSpace::Definite(size.width as f32),
                    height: taffy::AvailableSpace::Definite(size.height as f32),
                });
            }
        }
    }
    pub fn input_event(&mut self, event: InputEvent) {
        match self {
            LoadGame::NotLoaded => (),
            LoadGame::Game(game) => game.input_event(event),
            LoadGame::Error(gui, _) => {
                let (events, _) = gui.input_event(event);
                events.execute(gui);
            }
        }
    }
    pub fn update(&mut self, dt: f32) {
        if let LoadGame::Game(game) = self {
            game.update(dt);
        }
    }
    pub fn clear_color(&self) -> Rgba {
        match self {
            LoadGame::NotLoaded => Rgba::BLACK,
            LoadGame::Game(game) => game.clear_color(),
            LoadGame::Error(_, renderer) => renderer.background_color(),
        }
    }
    pub fn render(&mut self, context: &Context, pass: &mut wgpu::RenderPass) {
        match self {
            LoadGame::NotLoaded => (),
            LoadGame::Game(game) => game.render(context, pass),
            LoadGame::Error(gui, renderer) => renderer.render(context, pass, gui),
        }
    }
}
