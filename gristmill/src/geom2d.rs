use glam::{IVec2, Vec2};
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign};

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub const ZERO: Size = Size {
        width: 0,
        height: 0,
    };
    pub const fn new(width: u32, height: u32) -> Size {
        Size { width, height }
    }

    pub fn aspect(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn as_vec2(self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }
    pub fn as_ivec2(self) -> IVec2 {
        IVec2::new(self.width as i32, self.height as i32)
    }
}

impl Add for Size {
    type Output = Size;
    fn add(self, rhs: Self) -> Self::Output {
        Size::new(self.width + rhs.width, self.height + rhs.height)
    }
}
impl AddAssign for Size {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl From<[u32; 2]> for Size {
    fn from(size: [u32; 2]) -> Size {
        Size {
            width: size[0],
            height: size[1],
        }
    }
}

impl From<(u32, u32)> for Size {
    fn from(size: (u32, u32)) -> Size {
        Size {
            width: size.0,
            height: size.1,
        }
    }
}

impl From<IVec2> for Size {
    fn from(size: IVec2) -> Size {
        Size {
            width: size.x as u32,
            height: size.y as u32,
        }
    }
}

impl From<Size> for vulkano::image::ImageDimensions {
    fn from(size: Size) -> Self {
        vulkano::image::ImageDimensions::Dim2d {
            width: size.width,
            height: size.height,
            array_layers: 1,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct Rect {
    pub position: IVec2,
    pub size: Size,
}

impl Rect {
    pub const ZERO: Rect = Rect {
        position: IVec2::ZERO,
        size: Size::ZERO,
    };
    pub fn new(position: IVec2, size: Size) -> Rect {
        Rect { position, size }
    }
    pub fn from_size(size: Size) -> Rect {
        Rect {
            position: IVec2::ZERO,
            size,
        }
    }

    pub fn top_left(&self) -> IVec2 {
        self.position
    }
    pub fn top_right(&self) -> IVec2 {
        IVec2 {
            x: self.position.x + self.size.width as i32,
            y: self.position.y,
        }
    }
    pub fn bottom_left(&self) -> IVec2 {
        IVec2 {
            x: self.position.x,
            y: self.position.y + self.size.height as i32,
        }
    }
    pub fn bottom_right(&self) -> IVec2 {
        IVec2 {
            x: self.position.x + self.size.width as i32,
            y: self.position.y + self.size.height as i32,
        }
    }
    pub fn center(&self) -> IVec2 {
        IVec2 {
            x: self.position.x + (self.size.width / 2) as i32,
            y: self.position.y + (self.size.height / 2) as i32,
        }
    }

    pub fn contains(&self, point: IVec2) -> bool {
        self.position.x <= point.x
            && self.position.y <= point.y
            && self.position.x + self.size.width as i32 > point.x
            && self.position.y + self.size.height as i32 > point.y
    }

    pub fn inset(&self, insets: EdgeRect) -> Rect {
        let width = self.size.width as i32;
        let height = self.size.height as i32;
        let inset_width = insets.left + insets.right;
        let inset_height = insets.top + insets.bottom;
        Rect {
            position: IVec2 {
                x: self.position.x + insets.left,
                y: self.position.y + insets.right,
            },
            size: Size {
                width: if inset_width >= width {
                    0
                } else {
                    (width - inset_width) as u32
                },
                height: if inset_height >= height {
                    0
                } else {
                    (height - inset_height) as u32
                },
            },
        }
    }
}

impl Add<IVec2> for Rect {
    type Output = Rect;
    fn add(self, rhs: IVec2) -> Self::Output {
        Rect::new(self.position + rhs, self.size)
    }
}
impl AddAssign<IVec2> for Rect {
    fn add_assign(&mut self, other: IVec2) {
        *self = *self + other;
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct EdgeRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl EdgeRect {
    pub const ZERO: EdgeRect = EdgeRect {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        EdgeRect {
            left,
            top,
            right,
            bottom,
        }
    }
    pub fn splat(v: i32) -> Self {
        EdgeRect {
            left: v,
            top: v,
            right: v,
            bottom: v,
        }
    }
}
