use std::{
    cell::RefCell,
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use crate::Size;

pub type BufReader = std::io::BufReader<File>;
pub type BufWriter = std::io::BufWriter<File>;
use serde::Serialize;
pub use serde_yml::Value as YamlValue;

#[derive(Debug)]
enum ErrorKind {
    IoError(std::io::Error),
    Format,
    Other,
}

pub struct AssetError {
    path: PathBuf,
    write: bool,
    kind: ErrorKind,
    info: String,
}

impl AssetError {
    pub fn new_io(path: PathBuf, write: bool, error: std::io::Error) -> Self {
        AssetError {
            path,
            write,
            kind: ErrorKind::IoError(error),
            info: String::new(),
        }
    }
    pub fn new_format<E: ToString>(path: PathBuf, write: bool, error: E) -> Self {
        AssetError {
            path,
            write,
            kind: ErrorKind::Format,
            info: error.to_string(),
        }
    }
    fn new_png(path: PathBuf, error: png::DecodingError) -> Self {
        match error {
            png::DecodingError::IoError(error) => AssetError {
                path,
                write: false,
                kind: ErrorKind::IoError(error),
                info: String::new(),
            },
            png::DecodingError::Format(error) => AssetError {
                path,
                write: false,
                kind: ErrorKind::Format,
                info: error.to_string(),
            },
            _ => AssetError {
                path,
                write: false,
                kind: ErrorKind::Other,
                info: error.to_string(),
            },
        }
    }

    pub fn not_found(&self) -> bool {
        if let ErrorKind::IoError(error) = &self.kind {
            error.kind() == std::io::ErrorKind::NotFound
        } else {
            false
        }
    }
}

impl std::fmt::Debug for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let read_write = if self.write { "writing" } else { "reading" };
        write!(f, "Error {} {}: ", read_write, self.path.to_string_lossy())?;
        if let ErrorKind::IoError(error) = &self.kind {
            if self.write && error.kind() == std::io::ErrorKind::NotFound {
                // When a NotFound error occurs while writing, it means a parent directory doesn't exist.
                write!(f, "The parent directory does not exist.")?;
                if let Some(code) = error.raw_os_error() {
                    write!(f, " (os error {code})")?;
                }
                Ok(())
            } else {
                write!(f, "{}", error)
            }
        } else {
            write!(f, "{}", self.info)
        }
    }
}
impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::error::Error for AssetError {}

pub type Result<T> = std::result::Result<T, AssetError>;

pub trait Asset: Sized {
    fn load(path: &Path) -> Result<Self>;
}

pub fn set_cwd_from_executable() -> std::result::Result<(), std::io::Error> {
    let mut path = std::env::current_exe().expect("couldn't get executable path");
    path.pop();
    std::env::set_current_dir(path)
}

static BASE_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn set_base_path(base_path: &str) -> std::result::Result<(), PathBuf> {
    BASE_PATH.set(Path::new(base_path).to_path_buf())
}
pub fn file_path(path: &Path) -> PathBuf {
    let mut file_path = BASE_PATH.get().cloned().unwrap_or_default();
    file_path.push(path);
    file_path
}

pub fn load_file(path: &Path) -> Result<BufReader> {
    log::debug!("Load file: {}", path.display());
    let file =
        File::open(file_path(path)).map_err(|e| AssetError::new_io(path.to_owned(), false, e))?;
    Ok(BufReader::new(file))
}
pub fn save_file(path: &Path) -> Result<BufWriter> {
    log::debug!("Save file: {}", path.display());
    let file =
        File::create(file_path(path)).map_err(|e| AssetError::new_io(path.to_owned(), true, e))?;
    Ok(BufWriter::new(file))
}

thread_local! {
    static CURRENT_ASSET: RefCell<Vec<PathBuf>> = const { RefCell::new(Vec::new()) };
}

pub fn sub_asset_path(base_path: &str, relative: bool, path: &Path) -> PathBuf {
    let mut buf = Path::new(base_path).to_path_buf();
    if relative {
        let current = CURRENT_ASSET.with(|current_asset| {
            current_asset
                .borrow()
                .last()
                .and_then(|path| path.file_stem())
                .map(ToOwned::to_owned)
        });
        if let Some(current) = current {
            buf.push(current);
        } else {
            log::warn!("sub_asset_path: no current asset");
        }
    }
    buf.push(path);
    buf
}

#[macro_export]
macro_rules! impl_sub_asset {
    ($type:ident, $base_path:expr, $relative:expr) => {
        impl TryFrom<std::path::PathBuf> for $type {
            type Error = $crate::asset::AssetError;
            fn try_from(value: std::path::PathBuf) -> $crate::asset::Result<Self> {
                $crate::asset::Asset::load(&$crate::asset::sub_asset_path(
                    $base_path, $relative, &value,
                ))
                .map($type)
            }
        }
    };
}

pub trait YamlAsset: serde::de::DeserializeOwned {}

impl<T: YamlAsset> Asset for T {
    fn load(path: &Path) -> Result<Self> {
        let reader = load_file(path)?;
        CURRENT_ASSET.with(|current_asset| current_asset.borrow_mut().push(path.to_owned()));
        let result = serde_yml::from_reader(reader)
            .map_err(|e| AssetError::new_format(path.to_owned(), false, e));
        CURRENT_ASSET.with(|current_asset| current_asset.borrow_mut().pop());
        result
    }
}

pub trait YamlData: serde::de::DeserializeOwned {}

impl<T: YamlData> YamlAsset for Vec<T> {}
impl<T: YamlData> YamlAsset for HashMap<String, T> {}
impl<T: YamlData> YamlAsset for HashMap<String, Vec<T>> {}

pub fn save_yaml_file<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let writer = save_file(path)?;
    serde_yml::to_writer(writer, value)
        .map_err(|e| AssetError::new_format(path.to_owned(), true, e))
}

pub struct Image {
    pub size: Size,
    pub data: Vec<u8>,
}

impl Image {
    pub fn subimage(&self, x: u32, y: u32, size: Size) -> Image {
        let origin = (x + (y * size.width)) as usize * 4;
        let src_stride = self.size.width as usize * 4;
        let dst_stride = size.width as usize * 4;
        let mut subimage = Image { size, data: vec![0; (size.width * size.height) as usize * 4] };
        for row in 0..(size.height as usize) {
            let src_index = origin + (row * src_stride);
            let dst_index = row * dst_stride;
            let src = &self.data[src_index..src_index+dst_stride];
            let dst = &mut subimage.data[dst_index..dst_index+dst_stride];
            dst.copy_from_slice(src);
        }
        subimage
    }
}
impl Asset for Image {
    fn load(path: &Path) -> Result<Self> {
        use png::*;
        let reader = load_file(path)?;
        let mut decoder = Decoder::new(reader);
        decoder.set_transformations(Transformations::ALPHA);
        let mut image_reader = decoder
            .read_info()
            .map_err(|e| AssetError::new_png(path.to_owned(), e))?;
        let mut data = vec![0; image_reader.output_buffer_size()];
        let info = image_reader
            .next_frame(&mut data)
            .map_err(|e| AssetError::new_png(path.to_owned(), e))?;
        data.truncate(info.buffer_size());
        assert_eq!(info.bit_depth, BitDepth::Eight);
        match info.color_type {
            ColorType::Rgba => {}
            ColorType::GrayscaleAlpha => {
                data = data
                    .chunks_exact(2)
                    .flat_map(|x| [x[0], x[0], x[0], x[1]])
                    .collect();
            }
            _ => {
                return Err(AssetError::new_format(
                    path.to_owned(),
                    false,
                    format!("unsupported color type {:?}", info.color_type),
                ))
            }
        }
        Ok(Image {
            size: Size::new(info.width, info.height),
            data,
        })
    }
}
