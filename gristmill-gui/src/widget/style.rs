use gristmill::{
    asset::{Asset, AssetError, AssetResult, AssetWrite, BufReader, BufWriter},
    color::Pixel,
    color::{LinSrgb, WithAlpha},
    geom2d::Size,
    render::{
        texture::{Texture, TextureStorage},
        RenderContext,
    },
    Color,
};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use toml::value::{Array, Table, Value};

pub type StyleValue = Value;
pub type StyleValues = Table;

pub trait FromStyleValue: Sized {
    fn from_style(value: &Value) -> Option<Self>;
}

pub(crate) fn make_empty_texture() -> Value {
    let mut table = Table::new();
    table.insert("texture".to_owned(), Value::Boolean(false));
    Value::Table(table)
}
fn convert_i64_array(array: &Array) -> Option<Vec<i64>> {
    if array.iter().any(|x| !x.is_integer()) {
        None
    } else {
        Some(array.iter().map(|x| x.as_integer().unwrap()).collect())
    }
}
fn convert_f32_array(array: &Array) -> Option<Vec<f32>> {
    if array.iter().any(|x| !x.is_float()) {
        None
    } else {
        Some(array.iter().map(|x| x.as_float().unwrap() as f32).collect())
    }
}

impl FromStyleValue for i32 {
    fn from_style(value: &Value) -> Option<Self> {
        if let Value::Integer(int) = value {
            Some(*int as i32)
        } else {
            None
        }
    }
}
impl FromStyleValue for Size {
    fn from_style(value: &Value) -> Option<Self> {
        if let Value::Array(array) = value {
            if let Some(array) = convert_i64_array(array) {
                if array.len() == 2 && array[0] >= 0 && array[1] >= 0 {
                    return Some(Size::new(array[0] as u32, array[1] as u32));
                }
            }
        }
        None
    }
}
impl FromStyleValue for Color {
    fn from_style(value: &Value) -> Option<Self> {
        if let Value::Array(array) = value {
            if let Some(array) = convert_f32_array(array) {
                if array.len() == 3 {
                    return Some(LinSrgb::from_raw(&array[0..3]).with_alpha(1.0));
                } else if array.len() == 4 {
                    return Some(*Color::from_raw(&array[0..4]));
                }
            }
        }
        None
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WidgetStyles(toml::value::Table);

impl Asset for WidgetStyles {
    fn read_from(mut reader: BufReader) -> AssetResult<Self> {
        let mut string = String::new();
        reader.read_to_string(&mut string)?;
        toml::from_str(&string).map_err(|err| AssetError::InvalidFormat(err.to_string()))
    }
}

impl AssetWrite for WidgetStyles {
    fn write_to(value: &Self, mut writer: BufWriter) -> AssetResult<()> {
        let string =
            toml::to_string(value).map_err(|err| AssetError::InvalidFormat(err.to_string()))?;
        writer.write_all(string.as_bytes())?;
        Ok(())
    }
}

impl WidgetStyles {
    pub fn new() -> Self {
        Default::default()
    }
    pub(crate) fn with_all_defaults() -> Self {
        use super::*;
        let mut styles = Self::new();
        styles.insert(Image::class_name(), Image::default_style());
        styles.insert(Text::class_name(), Text::default_style());
        styles.insert(Button::class_name(), Button::default_style());
        styles
    }

    pub fn get(&self, class: &str) -> Option<&StyleValues> {
        self.0.get(class).and_then(|v| v.as_table())
    }
    pub fn insert(&mut self, class: &str, values: StyleValues) {
        self.0.insert(class.to_owned(), Value::Table(values));
    }

    pub fn load_textures(&self, context: &mut RenderContext) {
        let textures = TextureStorage::assets();
        for values in self.0.values().filter_map(|v| v.as_table()) {
            for value in values.values() {
                if let Value::Table(table) = value {
                    if let Some(Value::String(texture)) = table.get("texture") {
                        textures.load(context, texture);
                    }
                }
            }
        }
    }
    pub fn query<'a, I>(&'a self, classes: I) -> StyleQuery
    where
        I: IntoIterator<Item = &'a str>,
    {
        StyleQuery(
            classes
                .into_iter()
                .filter_map(|class| self.0.get(class))
                .filter_map(|v| v.as_table())
                .collect(),
        )
    }
}

pub struct StyleQuery<'a>(Vec<&'a StyleValues>);

impl<'a> StyleQuery<'a> {
    pub fn get<T>(&self, key: &str) -> Option<T>
    where
        T: FromStyleValue,
    {
        for values in self.0.iter() {
            if let Some(value) = values.get(key).and_then(|v| T::from_style(v)) {
                return Some(value);
            }
        }
        None
    }
    pub fn get_texture(&self, key: &str) -> Option<Texture> {
        for values in self.0.iter() {
            if let Some(Value::Table(table)) = values.get(key) {
                if let Some(Value::String(texture)) = table.get("texture") {
                    return TextureStorage::assets().get(texture);
                }
            }
        }
        None
    }
}
