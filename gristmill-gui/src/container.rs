use gristmill::geometry2d::*;
use super::layout::{Layout, Padding};

pub trait Container {
    fn layout_child(&mut self, rect: Rect, child_index: usize, child_size: Size) -> Layout;
}

// TODO start from any corner, not just top-left
pub struct FlowContainer {
    last_rect: Rect,
    line_height: u32,
    padding: Padding,
}

impl FlowContainer {
    pub fn new(padding: Padding) -> FlowContainer {
        FlowContainer { last_rect: Rect::zero(), line_height: 0, padding }
    }
}

impl Container for FlowContainer {
    fn layout_child(&mut self, rect: Rect, child_index: usize, child_size: Size) -> Layout {
        let child_position = if child_index == 0 {
            Point::new(self.padding.outside(), self.padding.outside())
        } else {
            Point {
                x: self.last_rect.top_right().x + self.padding.inside(),
                y: self.last_rect.position.y,
            }
        };
        let mut child_rect = Rect::new(child_position, child_size);
        if child_rect.position.x > 0 && child_rect.top_right().x > rect.size.width as i32 {
            child_rect.position.x = 0;
            child_rect.position.y += self.line_height as i32 + self.padding.inside();
            self.line_height = child_size.height;
        }
        else {
            self.line_height = u32::max(self.line_height, child_size.height);
        }
        self.last_rect = child_rect;
        Layout::offset_parent(child_rect)
    }
}

pub struct TableContainer {
    column_widths: Vec<u32>,
    total_container_width: u32,
    row_height: u32,
    padding: Padding,
    between_column_padding: Option<i32>,
    
    last_row: usize,
    last_rect: Rect,
}

impl TableContainer {
    pub fn new<'a, I>(columns: I, row_height: u32, padding: Padding, between_column_padding: Option<i32>) -> TableContainer where I: IntoIterator<Item=&'a u32> {
        let column_widths: Vec<_> = columns.into_iter().cloned().collect();
        let mut total_container_width = column_widths.iter().sum();
        let inside_padding = if let Some(padding) = between_column_padding { padding } else { padding.inside() };
        total_container_width = (total_container_width as i32 + (inside_padding * (column_widths.len() - 1) as i32) + (padding.outside() * 2)) as u32;
        TableContainer {
            column_widths,
            total_container_width,
            row_height,
            padding,
            between_column_padding,
            last_row: 0,
            last_rect: Rect::default(),
        }
    }
    fn column_width(&self, col: usize, total_width: u32) -> u32 {
        let width = self.column_widths[col];
        if width != 0 {
            width
        } else {
            // This column expands to fill all unused space
            if self.total_container_width > total_width { 0 }
            else {
                total_width - self.total_container_width
            }
        }
    }
}

impl Container for TableContainer {
    fn layout_child(&mut self, rect: Rect, child_index: usize, _child_size: Size) -> Layout {
        let num_cols = self.column_widths.len();
        let row = child_index / num_cols;
        let col = child_index % num_cols;
        let child_position = if row == 0 && col == 0 {
            Point::new(self.padding.outside(), self.padding.outside())
        } else if row == self.last_row {
            let last_top_right = self.last_rect.top_right();
            let padding = if let Some(padding) = self.between_column_padding { padding } else { self.padding.inside() };
            Point::new(last_top_right.x + padding, last_top_right.y)
        } else {
            let y = self.last_rect.bottom_left().y + self.padding.inside();
            Point::new(self.padding.outside(), y)
        };
        self.last_row = row;
        let child_rect = Rect::new(child_position, Size::new(self.column_width(col, rect.size.width), self.row_height));
        self.last_rect = child_rect;
        Layout::offset_parent(child_rect)
    }
}
