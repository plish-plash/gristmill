mod game_loop;
pub mod color;
pub mod input;
pub mod gui;
pub mod renderer;

// ------------------------------------------------------------------------------------------------

pub use renderer::Game;

pub fn read_ron_file<'a, T, P>(path: P) -> std::io::Result<T> where T: serde::de::DeserializeOwned, P: AsRef<std::path::Path> {
    ron::from_str(&std::fs::read_to_string(path)?).map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
}
