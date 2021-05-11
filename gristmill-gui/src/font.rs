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
        FONTS.as_ref().expect("error during font load")
    }
}

pub fn load_fonts(resources: &mut Resources) {
    init_logging();
    FONTS_INIT.call_once(|| {
        let asset_list = match AssetList::read("fonts") {
            Ok(value) => value,
            Err(error) => {
                log::error!("Failed to load font list: {}", error);
                return;
            }
        };
        if asset_list.loader() != "font" {
            log::error!("Invalid loader for font list (expected \"font\", got \"{}\")", asset_list.loader());
            return;
        }
        let mut font_store = FontStore { fonts: Vec::new() };
        let mut font_list = FontList { names: Vec::new() };
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

pub struct FontStore {
    fonts: Vec<FontAsset>,
}

impl FontStore {
    pub fn get(&self, index: Font) -> &rusttype::Font<'static> {
        &self.fonts[index.0].0
    }
}

pub struct FontList {
    names: Vec<String>,
}

impl FontList {
    pub fn find(&self, name: &str) -> Font {
        let index = self.names.iter().position(|n| n == name);
        Font(index.expect("unknown font"))
    }
}
