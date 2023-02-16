use gristmill_core::{
    asset::{self, AssetError, AssetResult},
    geom2d::EdgeRect,
    math::IVec2,
    Color,
};
use gristmill_render::{RenderContext, Texture};
use serde::Deserialize;
use std::collections::HashMap;

use crate::{Anchor, NodeLayout};

#[derive(Clone, Deserialize)]
#[serde(try_from = "toml::Value")]
pub enum StyleValue {
    String(String),
    Integer(i32),
    Float(f32),
    Boolean(bool),
    Texture(Option<Texture>),
    IntegerArray(Vec<i32>),
    FloatArray(Vec<f32>),
}

impl TryFrom<toml::Value> for StyleValue {
    type Error = &'static str;
    fn try_from(value: toml::Value) -> Result<Self, Self::Error> {
        match value {
            toml::Value::String(value) => Ok(StyleValue::String(value)),
            toml::Value::Integer(value) => Ok(StyleValue::Integer(value as i32)),
            toml::Value::Float(value) => Ok(StyleValue::Float(value as f32)),
            toml::Value::Boolean(value) => Ok(StyleValue::Boolean(value)),
            toml::Value::Datetime(_value) => Err("datetime not valid as style value"),
            toml::Value::Array(value) => {
                if value.iter().all(|x| x.is_integer()) {
                    Ok(StyleValue::IntegerArray(
                        value
                            .iter()
                            .map(|x| x.as_integer().unwrap() as i32)
                            .collect(),
                    ))
                } else if value.iter().all(|x| x.is_float()) {
                    Ok(StyleValue::FloatArray(
                        value.iter().map(|x| x.as_float().unwrap() as f32).collect(),
                    ))
                } else {
                    Err("style array must only contain ints or floats")
                }
            }
            toml::Value::Table(_value) => Err("table not valid as style value"),
        }
    }
}

impl TryFrom<StyleValue> for String {
    type Error = ();
    fn try_from(value: StyleValue) -> Result<Self, Self::Error> {
        if let StyleValue::String(value) = value {
            Ok(value)
        } else {
            Err(())
        }
    }
}
impl TryFrom<StyleValue> for i32 {
    type Error = ();
    fn try_from(value: StyleValue) -> Result<Self, Self::Error> {
        if let StyleValue::Integer(value) = value {
            Ok(value)
        } else {
            Err(())
        }
    }
}
impl TryFrom<StyleValue> for u32 {
    type Error = ();
    fn try_from(value: StyleValue) -> Result<Self, Self::Error> {
        if let StyleValue::Integer(value) = value {
            if value >= 0 {
                return Ok(value as u32);
            }
        }
        Err(())
    }
}
impl TryFrom<StyleValue> for Option<Texture> {
    type Error = ();
    fn try_from(value: StyleValue) -> Result<Self, Self::Error> {
        if let StyleValue::Texture(value) = value {
            Ok(value)
        } else {
            Err(())
        }
    }
}
impl TryFrom<StyleValue> for IVec2 {
    type Error = ();
    fn try_from(value: StyleValue) -> Result<Self, Self::Error> {
        if let StyleValue::IntegerArray(value) = value {
            if let [x, y] = value[..] {
                return Ok(IVec2 { x, y });
            }
        }
        Err(())
    }
}
impl TryFrom<StyleValue> for EdgeRect {
    type Error = ();
    fn try_from(value: StyleValue) -> Result<Self, Self::Error> {
        if let StyleValue::Integer(value) = value {
            return Ok(EdgeRect::splat(value));
        } else if let StyleValue::IntegerArray(value) = value {
            if let [top, right, bottom, left] = value[..] {
                return Ok(EdgeRect::new(top, right, bottom, left));
            }
        }
        Err(())
    }
}
impl TryFrom<StyleValue> for Color {
    type Error = ();
    fn try_from(value: StyleValue) -> Result<Self, Self::Error> {
        if let StyleValue::FloatArray(value) = value {
            match value[..] {
                [r, g, b] => return Ok(Color::new_opaque(r, g, b)),
                [r, g, b, a] => return Ok(Color::new(r, g, b, a)),
                _ => (),
            }
        }
        Err(())
    }
}
impl TryFrom<StyleValue> for bool {
    type Error = ();
    fn try_from(value: StyleValue) -> Result<Self, Self::Error> {
        String::try_from(value)?.parse().map_err(|_| ())
    }
}
impl TryFrom<StyleValue> for Anchor {
    type Error = ();
    fn try_from(value: StyleValue) -> Result<Self, Self::Error> {
        String::try_from(value)?.parse()
    }
}

pub type StyleValues = HashMap<String, StyleValue>;

pub trait WidgetStyle {
    fn widget_value<T: TryFrom<StyleValue>>(&mut self, key: &str, default: T) -> T;
    fn widget_layout(&mut self) -> NodeLayout;
}

impl WidgetStyle for StyleValues {
    fn widget_value<T: TryFrom<StyleValue>>(&mut self, key: &str, default: T) -> T {
        self.remove(key)
            .and_then(|val| val.try_into().ok())
            .unwrap_or(default)
    }
    fn widget_layout(&mut self) -> NodeLayout {
        let mut layout = NodeLayout {
            child_layout: self.widget_value("child_layout", String::new()),
            child_spacing: self.widget_value("child_spacing", 0),
            size: self.widget_value("size", IVec2::ZERO),
            margin: self.widget_value("margin", EdgeRect::ZERO),
            anchors: (
                self.widget_value("hanchor", Anchor::Begin),
                self.widget_value("vanchor", Anchor::Begin),
            ),
        };
        if self.contains_key("width") {
            layout.size.x = self.widget_value("width", 0);
        }
        if self.contains_key("height") {
            layout.size.y = self.widget_value("height", 0);
        }
        layout
    }
}

#[derive(Default)]
pub struct WidgetStyles(HashMap<String, StyleValues>);

impl WidgetStyles {
    pub fn load_asset(context: &mut RenderContext) -> AssetResult<Self> {
        let contents = asset::load_text_file("assets", "gui_styles.toml")?;
        let table =
            toml::from_str(&contents).map_err(|err| AssetError::InvalidFormat(err.to_string()))?;
        let mut styles = WidgetStyles(table);
        styles.load_textures(context)?;
        Ok(styles)
    }
    fn load_textures(&mut self, context: &mut RenderContext) -> AssetResult<()> {
        for group in self.0.values_mut() {
            for (key, value) in group.iter_mut() {
                if key.ends_with("texture") {
                    if let StyleValue::String(file) = value {
                        *value = StyleValue::Texture(Some(context.load_texture(file)?));
                    } else {
                        *value = StyleValue::Texture(None);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn query<'a, I>(&self, class: I) -> StyleValues
    where
        I: Iterator<Item = &'a str>,
    {
        StyleValues::from_iter(
            class
                .filter_map(|class| self.0.get(class))
                .flat_map(Clone::clone),
        )
    }
}
