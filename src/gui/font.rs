use std::collections::HashMap;
use rusttype::Font;

// Font objects more or less need a static lifetime. To layout text we use PositionedGlyph, which borrows the font, which is
// a huge headache if the font has any lifetime other than static. (Note that this is NOT the lifetime parameter for the Font type).
static mut FONTS: Option<FontStore> = None;

pub struct FontStore {
    fonts: HashMap<String, Font<'static>>,
    first_key: String,
}

impl FontStore {
    fn new() -> FontStore {
        FontStore { fonts: HashMap::new(), first_key: String::new() }
    }
    pub fn get(&self, key: Option<&str>) -> &Font<'static> {
        let key = key.unwrap_or(&self.first_key);
        self.fonts.get(key).expect("font does not exist")
    }
    pub fn load<P>(&mut self, key: &'static str, path: P) where P: AsRef<std::path::Path> {
        let font = Font::try_from_vec(std::fs::read(path).unwrap()).unwrap();
        if self.fonts.is_empty() {
            self.first_key = key.to_string();
        }
        self.fonts.insert(key.to_string(), font);
    }
}

// TODO make these unsafe blocks watertight.
// also error handling

pub fn load_fonts<F>(f: F) where F: FnOnce(&mut FontStore) {
    // Potentially unsound if another thread is also executing load_fonts (which should never happen but still).
    unsafe {
        if FONTS.is_some() {
            panic!("fonts have already been loaded!");
        }
    }
    let mut font_store = FontStore::new();
    f(&mut font_store);
    // This is safe because it only ever happens once, and
    // FONTS is None meaning there can't be any dangling references to it.
    unsafe {
        FONTS = Some(font_store);
    }
}

pub fn fonts() -> &'static FontStore {
    // This is only safe if load_fonts isn't executing in another thread.
    unsafe {
        FONTS.as_ref().expect("no fonts loaded")
    }
}
