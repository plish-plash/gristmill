pub mod asset;
pub mod geom2d;
pub mod input;

pub use glam as math;
pub use slotmap;

#[derive(Copy, Clone, Debug)]
pub struct Color([f32; 4]);

impl Color {
    pub const WHITE: Color = Color([1., 1., 1., 1.]);
    pub const BLACK: Color = Color([0., 0., 0., 1.]);
    pub const RED: Color = Color([1., 0., 0., 1.]);
    pub const GREEN: Color = Color([0., 1., 0., 1.]);
    pub const BLUE: Color = Color([0., 0., 1., 1.]);

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color([r, g, b, a])
    }
    pub fn new_opaque(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }
    pub fn new_value(value: f32) -> Self {
        Self::new(value, value, value, 1.0)
    }
}

impl From<[f32; 4]> for Color {
    fn from(color: [f32; 4]) -> Self {
        Color(color)
    }
}
impl From<Color> for [f32; 4] {
    fn from(color: Color) -> Self {
        color.0
    }
}

#[macro_export]
macro_rules! new_storage_types {
    (type $storage_ty:ident = < $key_ty:ident , $value_ty:ty > ) => {
        $crate::slotmap::new_key_type! { struct $key_ty; }
        type $storage_ty = $crate::slotmap::SlotMap<$key_ty, $value_ty>;
    };
    (type $storage_ty:ident = $map_ty:ident < $key_ty:ident , $value_ty:ty > ) => {
        $crate::slotmap::new_key_type! { struct $key_ty; }
        type $storage_ty = $crate::slotmap::$map_ty<$key_ty, $value_ty>;
    };
    (pub type $storage_ty:ident = < $key_ty:ident , $value_ty:ty > ) => {
        $crate::slotmap::new_key_type! { pub struct $key_ty; }
        pub type $storage_ty = $crate::slotmap::SlotMap<$key_ty, $value_ty>;
    };
    (pub type $storage_ty:ident = $map_ty:ident < $key_ty:ident , $value_ty:ty > ) => {
        $crate::slotmap::new_key_type! { struct $key_ty; }
        pub type $storage_ty = $crate::slotmap::$map_ty<$key_ty, $value_ty>;
    };
}
