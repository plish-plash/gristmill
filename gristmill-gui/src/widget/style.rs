use gristmill::{
    asset::{Asset, AssetResult, AssetWrite, BufReader, BufWriter},
    color::Pixel,
    geom2d::{EdgeRect, Rect, Size},
    Color,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum StyleValue {
    Number(i32),
    Color([f32; 4]),
    Size(Size),
    Rect(Rect),
    EdgeRect(EdgeRect),
}

impl TryFrom<StyleValue> for i32 {
    type Error = ();
    fn try_from(value: StyleValue) -> Result<Self, Self::Error> {
        match value {
            StyleValue::Number(num) => Ok(num),
            _ => Err(()),
        }
    }
}
impl TryFrom<StyleValue> for Color {
    type Error = ();
    fn try_from(value: StyleValue) -> Result<Self, Self::Error> {
        match value {
            StyleValue::Color(color) => Ok(*Color::from_raw(&color)),
            _ => Err(()),
        }
    }
}
impl TryFrom<StyleValue> for Size {
    type Error = ();
    fn try_from(value: StyleValue) -> Result<Self, Self::Error> {
        match value {
            StyleValue::Number(num) => Ok(Size::new(num as u32, num as u32)),
            StyleValue::Size(size) => Ok(size),
            _ => Err(()),
        }
    }
}
impl TryFrom<StyleValue> for Rect {
    type Error = ();
    fn try_from(value: StyleValue) -> Result<Self, Self::Error> {
        match value {
            StyleValue::Rect(rect) => Ok(rect),
            _ => Err(()),
        }
    }
}
impl TryFrom<StyleValue> for EdgeRect {
    type Error = ();
    fn try_from(value: StyleValue) -> Result<Self, Self::Error> {
        match value {
            StyleValue::Number(num) => Ok(EdgeRect::splat(num)),
            StyleValue::EdgeRect(rect) => Ok(rect),
            _ => Err(()),
        }
    }
}

impl From<i32> for StyleValue {
    fn from(val: i32) -> StyleValue {
        StyleValue::Number(val)
    }
}
impl From<Color> for StyleValue {
    fn from(val: Color) -> StyleValue {
        StyleValue::Color(val.into_raw())
    }
}
impl From<Size> for StyleValue {
    fn from(val: Size) -> StyleValue {
        StyleValue::Size(val)
    }
}
impl From<Rect> for StyleValue {
    fn from(val: Rect) -> StyleValue {
        StyleValue::Rect(val)
    }
}
impl From<EdgeRect> for StyleValue {
    fn from(val: EdgeRect) -> StyleValue {
        StyleValue::EdgeRect(val)
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StyleValues(HashMap<String, StyleValue>);

impl StyleValues {
    pub fn new() -> StyleValues {
        Default::default()
    }
    pub fn get(&self, key: &str) -> Option<StyleValue> {
        self.0.get(key).cloned()
    }
    pub fn set<V: Into<StyleValue>>(&mut self, key: &str, value: V) -> &mut StyleValues {
        self.0.insert(key.to_owned(), value.into());
        self
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WidgetStyles(HashMap<String, StyleValues>);

impl Asset for WidgetStyles {
    fn read_from(reader: BufReader) -> AssetResult<Self> {
        gristmill::asset::util::read_ron(reader)
    }
}

impl AssetWrite for WidgetStyles {
    fn write_to(value: &Self, writer: BufWriter) -> AssetResult<()> {
        gristmill::asset::util::write_ron(writer, value)
    }
}

impl WidgetStyles {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn with_all_defaults() -> Self {
        use super::*;
        let mut styles = Self::new();
        styles.insert(Image::class_name(), Image::default_style());
        styles.insert(Text::class_name(), Text::default_style());
        styles.insert(Button::class_name(), Button::default_style());
        styles
    }

    pub fn get(&self, class: &str) -> Option<&StyleValues> {
        self.0.get(class)
    }
    pub fn insert(&mut self, class: &str, values: StyleValues) {
        self.0.insert(class.to_owned(), values);
    }

    pub fn query<'a, I>(&self, classes: I) -> StyleQuery
    where
        I: IntoIterator<Item = &'a str>,
    {
        StyleQuery(
            classes
                .into_iter()
                .filter_map(|class| self.0.get(class))
                .collect(),
        )
    }
}

pub struct StyleQuery<'a>(Vec<&'a StyleValues>);

impl<'a> StyleQuery<'a> {
    pub fn get<T>(&self, key: &str, default: T) -> T
    where
        T: TryFrom<StyleValue>,
    {
        for values in self.0.iter() {
            if let Some(value) = values.get(key).and_then(|v| T::try_from(v).ok()) {
                return value;
            }
        }
        default
    }
}
