// Utilities for working with stretch::Style

use stretch::geometry::Rect;
pub use stretch::style::*;

pub fn points_rect(points: f32) -> Rect<Dimension> {
    Rect {
        start: Dimension::Points(points),
        end: Dimension::Points(points),
        top: Dimension::Points(points),
        bottom: Dimension::Points(points),
    }
}
