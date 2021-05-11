use std::sync::Once;

use gristmill::asset::{Asset, AssetExt, AssetResult, AssetError, category, Resources, resource::AssetList};

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

pub fn load_fonts(resources: &mut Resources) {
    FONTS_INIT.call_once(|| {
        // TODO error handling
        let asset_list = AssetList::read("fonts").unwrap();
        if asset_list.loader() != "font" {
            panic!("invalid loader for font list");
        }
        let mut font_store = FontStore { fonts: Vec::new() };
        let mut font_list = FontList { names: Vec::new() };
        for item in asset_list {
            if item.asset_type != "font" {
                panic!("unexpected asset type in font list");
            }
            font_store.fonts.push(FontAsset::read(&item.asset_path).unwrap());
            font_list.names.push(item.name);
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
