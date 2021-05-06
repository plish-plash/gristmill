use gristmill::geometry2d::*;
use super::layout::Layout;

pub trait Container {
    fn layout_child(&mut self, rect: Rect, child_index: usize, child_size: Size) -> Layout;
}

// TODO start from any corner, not just top-left
pub struct FlowContainer {
    last_rect: Rect,
    line_height: u32,
    padding: i32,
    pad_outside: bool,
}

impl FlowContainer {
    pub fn new(padding: i32) -> FlowContainer {
        FlowContainer { last_rect: Rect::zero(), line_height: 0, padding, pad_outside: false }
    }
}

impl Container for FlowContainer {
    fn layout_child(&mut self, rect: Rect, child_index: usize, child_size: Size) -> Layout {
        let child_position = if child_index == 0 {
            if self.pad_outside { Point::new(self.padding, self.padding) } else { Point::origin() }
        } else {
            Point {
                x: self.last_rect.top_right().x + self.padding,
                y: self.last_rect.position.y,
            }
        };
        let mut child_rect = Rect::new(child_position, child_size);
        if child_rect.position.x > 0 && child_rect.top_right().x > rect.size.width as i32 {
            child_rect.position.x = 0;
            child_rect.position.y += self.line_height as i32 + self.padding;
            self.line_height = child_size.height;
        }
        else {
            self.line_height = u32::max(self.line_height, child_size.height);
        }
        self.last_rect = child_rect;
        Layout::offset_parent(child_rect)
    }
}
