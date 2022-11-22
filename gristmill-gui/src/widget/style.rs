use crate::widget::WidgetType;
use gristmill::color::Pixel;
use gristmill::geom2d::{EdgeRect, Rect, Size};
use gristmill::Color;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Borrow,
    collections::HashMap,
    hash::{Hash, Hasher},
    str::FromStr,
};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum StyleValue {
    Number(i32),
    Color([f32; 4]),
    Size(Size),
    Rect(Rect),
    EdgeRect(EdgeRect),
}

impl StyleValue {
    pub fn to_i32(self) -> Option<i32> {
        match self {
            StyleValue::Number(num) => Some(num),
            _ => None,
        }
    }
    pub fn to_color(self) -> Option<Color> {
        match self {
            StyleValue::Color(color) => Some(*Color::from_raw(&color)),
            _ => None,
        }
    }
    pub fn to_size(self) -> Option<Size> {
        match self {
            StyleValue::Number(num) => Some(Size::new(num as u32, num as u32)),
            StyleValue::Size(size) => Some(size),
            _ => None,
        }
    }
    pub fn to_rect(self) -> Option<Rect> {
        match self {
            StyleValue::Rect(rect) => Some(rect),
            _ => None,
        }
    }
    pub fn to_edge_rect(self) -> Option<EdgeRect> {
        match self {
            StyleValue::Number(num) => Some(EdgeRect::splat(num)),
            StyleValue::EdgeRect(rect) => Some(rect),
            _ => None,
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
        StyleValues(HashMap::new())
    }
    pub fn get(&self, key: &str) -> Option<StyleValue> {
        self.0.get(key).cloned()
    }
    pub fn set<V: Into<StyleValue>>(&mut self, key: &str, value: V) -> &mut StyleValues {
        self.0.insert(key.to_owned(), value.into());
        self
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct OwnedStyleRule {
    widget: String,
    class: Option<String>,
}

impl OwnedStyleRule {
    pub fn new(widget: WidgetType, class: Option<&str>) -> OwnedStyleRule {
        OwnedStyleRule {
            widget: widget.0.to_owned(),
            class: class.map(|s| s.to_owned()),
        }
    }
    pub fn as_ref(&self) -> StyleRule {
        StyleRule {
            widget: &self.widget,
            class: self.class.as_deref(),
        }
    }
}
impl FromStr for OwnedStyleRule {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((widget, class)) = s.split_once('.') {
            Ok(OwnedStyleRule {
                widget: widget.to_owned(),
                class: Some(class.to_owned()),
            })
        } else {
            Ok(OwnedStyleRule {
                widget: s.to_owned(),
                class: None,
            })
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct StyleRule<'a> {
    widget: &'a str,
    class: Option<&'a str>,
}

impl<'a> StyleRule<'a> {
    pub fn new(widget: WidgetType, class: Option<&'a str>) -> StyleRule<'a> {
        StyleRule {
            widget: widget.0,
            class,
        }
    }
}

pub trait Key {
    fn key(&self) -> StyleRule;
}

impl Key for OwnedStyleRule {
    fn key(&self) -> StyleRule {
        self.as_ref()
    }
}

impl<'a> Key for StyleRule<'a> {
    fn key(&self) -> StyleRule {
        *self
    }
}

impl<'a> Borrow<dyn Key + 'a> for OwnedStyleRule {
    fn borrow(&self) -> &(dyn Key + 'a) {
        self
    }
}

impl<'a> PartialEq for (dyn Key + 'a) {
    fn eq(&self, other: &Self) -> bool {
        self.key().eq(&other.key())
    }
}

impl<'a> Eq for (dyn Key + 'a) {}

impl<'a> Hash for (dyn Key + 'a) {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key().hash(state)
    }
}
