use std::{any::Any, collections::HashMap, path::Path, sync::RwLock};

use emath::{Align2, Vec2};
use serde::{de::DeserializeOwned, Deserialize};

use crate::{
    asset::{Asset, AssetError, YamlAsset, YamlValue},
    color::Color,
    gui::Padding,
};

type StyleValue = Box<dyn Any + Send + Sync>;

static STYLE: RwLock<Option<HashMap<String, HashMap<String, StyleValue>>>> = RwLock::new(None);

pub fn style_or<T: Clone + 'static>(class: &str, field: &str, default: T) -> T {
    let global_style = STYLE.read().unwrap();
    if let Some(global_style) = global_style.as_ref() {
        if let Some(value) = global_style
            .get(class)
            .and_then(|class_fields| class_fields.get(field))
        {
            if let Some(value) = value.downcast_ref::<T>() {
                return value.clone();
            } else {
                log::warn!("Wrong type for style field '{}:{}'", class, field);
            }
        }
    } else {
        log::warn!("Global style not set");
    }
    default
}
pub fn style<T: Default + Clone + 'static>(class: &str, field: &str) -> T {
    style_or(class, field, Default::default())
}

#[derive(Deserialize)]
#[serde(transparent)]
struct StyleSheetAsset(HashMap<String, HashMap<String, YamlValue>>);

impl YamlAsset for StyleSheetAsset {}

pub struct StyleSheet(HashMap<String, Box<dyn Fn(YamlValue) -> Result<StyleValue, String>>>);

impl StyleSheet {
    pub fn new_empty() -> Self {
        StyleSheet(HashMap::new())
    }
    pub fn add_field<T>(&mut self, field: &str)
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        fn to_box<T: Send + Sync + 'static>(value: T) -> StyleValue {
            Box::new(value)
        }
        self.0.insert(
            field.to_string(),
            Box::new(|value| {
                serde_yml::from_value::<T>(value)
                    .map(to_box)
                    .map_err(|e| e.to_string())
            }),
        );
    }

    pub fn load_global(&self, path: &Path) -> Result<(), AssetError> {
        fn tuple_result<A, B, E>(x: (A, Result<B, E>)) -> Result<(A, B), E> {
            match x.1 {
                Ok(v) => Ok((x.0, v)),
                Err(e) => Err(e),
            }
        }
        let field_de = |class: &'_ str, (field, value)| {
            let value = self
                .0
                .get(&field)
                .ok_or_else(|| "unknown field".to_string())
                .and_then(|f| f(value))
                .map_err(|e| format!("{}:{}: {}", class, field, e));
            (field, value)
        };
        let style_values = StyleSheetAsset::load(path)?;
        let style_values: Result<_, _> = style_values
            .0
            .into_iter()
            .map(|(class, fields)| {
                let fields = fields
                    .into_iter()
                    .map(|field| tuple_result(field_de(&class, field)))
                    .collect();
                tuple_result((class, fields))
            })
            .collect();
        let style_values =
            style_values.map_err(|e| AssetError::new_format(path.to_owned(), false, e))?;
        *STYLE.write().unwrap() = Some(style_values);
        Ok(())
    }
}
impl Default for StyleSheet {
    fn default() -> Self {
        let mut style = StyleSheet::new_empty();
        style.add_field::<Color>("color");
        style.add_field::<Vec2>("size");
        style.add_field::<bool>("grow");
        style.add_field::<Padding>("padding");
        style.add_field::<usize>("font-id");
        style.add_field::<f32>("font-scale");
        style.add_field::<Align2>("align");
        style.add_field::<bool>("wrap");
        style
    }
}
