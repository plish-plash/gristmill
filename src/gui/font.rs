use std::sync::Once;
use std::path::{Path, PathBuf};

use crate::asset::{Asset, AssetCategory, AssetResult, AssetError, load_asset};

// -------------------------------------------------------------------------------------------------

// Font objects more or less need a static lifetime. To layout text we use PositionedGlyph, which borrows the font, which is
// a huge headache if the font has any lifetime other than static. (Note that this is NOT the lifetime parameter for the Font type).
static mut FONTS: Option<FontStore> = None;
static FONTS_INIT: Once = Once::new();

pub fn fonts() -> &'static FontStore {
    if !FONTS_INIT.is_completed() {
        panic!("fonts have not been loaded");
    }
    unsafe {
        FONTS.as_ref().unwrap()
    }
}

pub fn load_fonts(asset_paths: Vec<String>) {
    FONTS_INIT.call_once(|| {
        // TODO error handling
        let fonts: Vec<FontAsset> = asset_paths.iter().map(|path|
            load_asset(path)
        ).collect::<Result<_, _>>().unwrap();
        unsafe {
            FONTS = Some(FontStore::new(fonts));
        }
    });
}

// -------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Default, Eq, PartialEq, Debug)]
pub struct Font(usize);

pub struct FontStore {
    fonts: Vec<FontAsset>,
}

impl FontStore {
    fn new(fonts: Vec<FontAsset>) -> FontStore {
        FontStore { fonts }
    }

    pub fn get(&self, index: Font) -> &rusttype::Font<'static> {
        &self.fonts[index.0].font
    }
    pub fn find_by_name(&self, name: &str) -> Option<Font> {
        for (index, font) in self.fonts.iter().enumerate() {
            if font.name == name {
                return Some(Font(index));
            }
        }
        None
    }
    pub fn find_by_path(&self, path: &Path) -> Option<Font> {
        for (index, font) in self.fonts.iter().enumerate() {
            if font.file_path == path {
                return Some(Font(index));
            }
        }
        None
    }
}

struct FontAsset {
    font: rusttype::Font<'static>,
    name: String,
    file_path: PathBuf,
}

impl Asset for FontAsset {
    fn category() -> AssetCategory { AssetCategory::Asset }
    fn file_extension() -> &'static str { "ttf" }
    fn load(file_path: PathBuf) -> AssetResult<Self> {
        let font = match rusttype::Font::try_from_vec(std::fs::read(&file_path)?) {
            Some(f) => f,
            None => return Err(AssetError::new_data()),
        };
        Ok(FontAsset {
            font,
            name: file_path.file_stem().unwrap().to_string_lossy().into_owned(),
            file_path,
        })
    }
}
