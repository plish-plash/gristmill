use std::{collections::HashMap, path::Path, sync::OnceLock};

use serde::Deserialize;

use crate::asset::{Asset, AssetError, YamlAsset};

#[derive(Deserialize)]
#[serde(transparent)]
pub struct Translations(pub HashMap<String, String>);
impl YamlAsset for Translations {}

static TRANSLATIONS: OnceLock<Translations> = OnceLock::new();

pub fn load_translations() -> Result<(), AssetError> {
    // TODO use correct locale
    let translations = Translations::load(Path::new("lang/en.yaml"))?;
    TRANSLATIONS.get_or_init(|| translations);
    Ok(())
}

pub fn tr(key: &str) -> &str {
    let translations = TRANSLATIONS.get().expect("translations not loaded");
    if let Some(value) = translations.0.get(key) {
        value
    } else {
        log::error!("Missing translation for {}", key);
        key
    }
}
