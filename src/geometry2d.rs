
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

    // pub fn offset(&mut self, other: Point) {
    //     self.x += other.x;
    //     self.y += other.y;
    // }
    pub fn relative_to(self, other: Point) -> Point {
        Point { x: self.x - other.x, y: self.y - other.y }
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

// impl From<Size> for stretch::geometry::Size<stretch::number::Number> {
//     fn from(size: Size) -> stretch::geometry::Size<stretch::number::Number> {
//         stretch::geometry::Size {
//             width: stretch::number::Number::Defined(size.width as f32),
//             height: stretch::number::Number::Defined(size.height as f32),
//         }
//     }
// }
// impl From<Size> for stretch::geometry::Size<stretch::style::Dimension> {
//     fn from(size: Size) -> stretch::geometry::Size<stretch::style::Dimension> {
//         stretch::geometry::Size {
//             width: stretch::style::Dimension::Points(size.width as f32),
//             height: stretch::style::Dimension::Points(size.height as f32),
//         }
//     }
// }

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
}

// impl From<stretch::result::Layout> for Rect {
//     fn from(layout: stretch::result::Layout) -> Rect {
//         let x = layout.location.x as i32;
//         let y = layout.location.y as i32;
//         let width = layout.size.width as u32;
//         let height = layout.size.height as u32;
//         Rect {
//             position: Point { x, y },
//             size: Size { width, height },
//         }
//     }
// }
