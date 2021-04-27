use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }
    pub fn nearest(x: f32, y: f32) -> Point {
        Point {
            x: x.round() as i32,
            y: y.round() as i32,
        }
    }
    pub fn origin() -> Point { Self::default() }

    pub fn relative_to(self, other: Point) -> Point {
        Point { x: self.x - other.x, y: self.y - other.y }
    }

    pub fn normalize_components(self, area_size: Size) -> [f32; 2] {
        [
            self.x as f32 / area_size.width as f32,
            self.y as f32 / area_size.height as f32,
        ]
    }
}

impl From<Point> for [f32; 2] {
    fn from(point: Point) -> [f32; 2] {
        [point.x as f32, point.y as f32]
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Size {
        Size { width, height }
    }
    pub fn zero() -> Size { Self::default() }

    pub fn aspect(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}

impl From<[u32; 2]> for Size {
    fn from(size: [u32; 2]) -> Size {
        Size { width: size[0], height: size[1] }
    }
}

impl From<Size> for [f32; 2] {
    fn from(size: Size) -> [f32; 2] {
        [size.width as f32, size.height as f32]
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Rect {
    pub position: Point,
    pub size: Size,
}

impl Rect {
    pub fn zero() -> Rect { Self::default() }

    pub fn top_left(&self) -> Point {
        self.position
    }
    pub fn top_right(&self) -> Point {
        Point { x: self.position.x + self.size.width as i32, y: self.position.y }
    }
    pub fn bottom_left(&self) -> Point {
        Point { x: self.position.x, y: self.position.y + self.size.height as i32 }
    }
    pub fn bottom_right(&self) -> Point {
        Point { x: self.position.x + self.size.width as i32, y: self.position.y + self.size.height as i32 }
    }

    pub fn contains(&self, point: Point) -> bool {
        self.position.x <= point.x &&
        self.position.y <= point.y &&
        self.position.x + self.size.width as i32 > point.x &&
        self.position.y + self.size.height as i32 > point.y
    }

    pub fn inset(&self, insets: EdgeRect) -> Rect {
        let width = self.size.width as i32;
        let height = self.size.height as i32;
        let inset_width = insets.left + insets.right;
        let inset_height = insets.top + insets.bottom;
        Rect {
            position: Point { x: self.position.x + insets.left, y: self.position.y + insets.right },
            size: Size {
                width: if inset_width >= width { 0 } else { (width - inset_width) as u32 },
                height: if inset_height >= height { 0 } else { (height - inset_height) as u32 },
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct EdgeRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}
