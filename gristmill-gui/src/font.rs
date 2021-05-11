use std::sync::Once;

use gristmill::init_logging;
use gristmill::asset::{Asset, AssetExt, AssetResult, AssetError, category, Resources, resource::AssetList};

// -------------------------------------------------------------------------------------------------

// Font objects more or less need a static lifetime. To layout text we use PositionedGlyph, which borrows the font, which is
// a huge headache if the font has any lifetime other than static. (Note that this is NOT the lifetime parameter for the Font type).
static mut FONTS: Option<FontStore> = None;
static FONTS_INIT: Once = Once::new();

pub fn fonts() -> &'static FontStore {
    if !FONTS_INIT.is_completed() {
        panic!("no fonts have been loaded");
    }
    unsafe {
        FONTS.as_ref().unwrap()
    }
}

fn read_fonts() -> (FontStore, FontList) {
    let mut font_store = FontStore { fonts: Vec::new() };
    let mut font_list = FontList { names: Vec::new() };
    let asset_list = match AssetList::read("fonts") {
        Ok(value) => value,
        Err(error) => {
            log::error!("Failed to load font list: {}", error);
            return (font_store, font_list);
        }
    };
    if asset_list.loader() != "font" {
        log::error!("Invalid loader for font list (expected \"font\", got \"{}\")", asset_list.loader());
        return (font_store, font_list);
    }
    for item in asset_list {
        if item.asset_type != "font" {
            log::warn!("Invalid asset type in font list (expected \"font\", got \"{}\")", item.asset_type);
            continue;
        }
        match FontAsset::read(&item.asset_path) {
            Ok(font) => {
                font_store.fonts.push(font);
                font_list.names.push(item.name);
            }
            Err(error) => log::error!("Failed to load font {}: {}", item.asset_path, error)
        }
    }
    (font_store, font_list)
}
pub fn load_fonts(resources: &mut Resources) {
    init_logging();
    FONTS_INIT.call_once(|| {
        let (font_store, font_list) = read_fonts();
        resources.insert("fonts", font_list);
        unsafe {
            FONTS = Some(font_store);
        }
    });
}

// -------------------------------------------------------------------------------------------------

struct FontAsset(rusttype::Font<'static>);

impl Asset for FontAsset {
    type Category = category::Data;
    fn read(asset_path: &str) -> AssetResult<Self> {
        let file_path = Self::get_file(asset_path, "ttf");
        log::trace!("Opening file {}", file_path.to_string_lossy());
        let font = match rusttype::Font::try_from_vec(std::fs::read(&file_path)?) {
            Some(f) => f,
            None => return Err(AssetError::InvalidData),
        };
        Ok(FontAsset(font))
    }
}

#[derive(Copy, Clone, Default, Eq, PartialEq, Debug)]
pub struct Font(usize);

impl Font {
    pub fn null() -> Font { Font(usize::MAX) }
    pub fn is_null(&self) -> bool { self.0 == usize::MAX }
}

pub struct FontStore {
    fonts: Vec<FontAsset>,
}

impl FontStore {
    pub fn get(&self, index: Font) -> Option<&rusttype::Font<'static>> {
        self.fonts.get(index.0).map(|asset| &asset.0)
    }
}

pub struct FontList {
    names: Vec<String>,
}

impl FontList {
    pub fn find(&self, name: &str) -> Font {
        let index = self.names.iter().position(|n| n == name);
        index.map(Font).unwrap_or_else(|| {
            log::warn!("Unknown font {}", name);
            Font::null()
        })
    }
}
