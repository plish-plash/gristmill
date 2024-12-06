use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub type Palette = HashMap<String, Color>;

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const ERROR: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub fn new_rgb(r: f32, g: f32, b: f32) -> Self {
        Color { r, g, b, a: 1.0 }
    }
    pub fn new_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }
    pub fn from_palette(palette: &Palette, name: &str) -> Self {
        palette.get(name).cloned().unwrap_or(Self::ERROR)
    }
}

pub fn default_gui_palette() -> Palette {
    let mut palette = Palette::new();
    palette.insert("button_normal".into(), Color::new_rgb(0.5, 0.5, 0.5));
    palette.insert("button_hover".into(), Color::new_rgb(0.6, 0.6, 0.6));
    palette.insert("button_press".into(), Color::new_rgb(0.4, 0.4, 0.4));
    palette.insert("button_disable".into(), Color::new_rgba(0.5, 0.5, 0.5, 0.5));
    palette
}
