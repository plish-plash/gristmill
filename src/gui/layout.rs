use std::convert::TryFrom;

use crate::geometry2d::*;

use super::{GuiNode, LayoutContext};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Side {
    Left, Right, Top, Bottom
}

impl Side {
    fn opposite(self) -> Side {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
            Side::Top => Side::Bottom,
            Side::Bottom => Side::Top,
        }
    }
    fn to_index(self) -> usize {
        match self {
            Side::Left => 0,
            Side::Right => 1,
            Side::Top => 2,
            Side::Bottom => 3,
        }
    }

    fn rect_edge(self, rect: Rect) -> i32 {
        match self {
            Side::Left => rect.position.x,
            Side::Right => rect.position.x + rect.size.width as i32,
            Side::Top => rect.position.y,
            Side::Bottom => rect.position.y + rect.size.height as i32,
        }
    }
}

pub enum AnchorTarget {
    None, Parent, PreviousSibling
}

pub enum AnchorTargetSide {
    SameSide, OppositeSide, Center
}

pub struct Anchor {
    pub target: AnchorTarget,
    pub target_side: AnchorTargetSide,
    pub offset: i32,
}

impl Default for Anchor {
    fn default() -> Self {
        Anchor {
            target: AnchorTarget::None,
            target_side: AnchorTargetSide::SameSide,
            offset: 0,
        }
    }
}

#[derive(Default)]
pub struct Layout {
    base_size: Size,
    anchors: [Anchor; 4],
}

impl Layout {
    pub fn with_base_size(base_size: Size) -> Layout {
        Layout {
            base_size, anchors: Default::default()
        }
    }
    pub fn set_anchor(&mut self, side: Side, anchor: Anchor) {
        self.anchors[side.to_index()] = anchor;
    }
}

impl Layout {
    fn get_edge(&self, context: &LayoutContext, node: GuiNode, side: Side) -> Option<i32> {
        let anchor = &self.anchors[side.to_index()];
        let target_rect = match anchor.target {
            AnchorTarget::None => return None,
            AnchorTarget::Parent => context.get_parent_rect(node),
            AnchorTarget::PreviousSibling => context.get_previous_sibling_rect(node),
        };
        let edge = match anchor.target_side {
            AnchorTargetSide::SameSide => side.rect_edge(target_rect),
            AnchorTargetSide::OppositeSide => side.opposite().rect_edge(target_rect),
            AnchorTargetSide::Center => (side.rect_edge(target_rect) + side.opposite().rect_edge(target_rect)) / 2,
        };
        let mut offset = anchor.offset;
        if side == Side::Right || side == Side::Bottom {
            offset = -offset;
        }
        Some(edge + offset)
    }

    pub fn layout_before_children(&self, context: &LayoutContext, node: GuiNode, parent_position: Point) -> Rect {
        let mut left_edge = self.get_edge(context, node, Side::Left);
        let mut right_edge = self.get_edge(context, node, Side::Right);
        let mut top_edge = self.get_edge(context, node, Side::Top);
        let mut bottom_edge = self.get_edge(context, node, Side::Bottom);
        if left_edge.is_some() && right_edge.is_none() {
            right_edge = Some(left_edge.unwrap() + self.base_size.width as i32);
        }
        else if right_edge.is_some() && left_edge.is_none() {
            left_edge = Some(right_edge.unwrap() - self.base_size.width as i32);
        }
        if top_edge.is_some() && bottom_edge.is_none() {
            bottom_edge = Some(top_edge.unwrap() + self.base_size.height as i32);
        }
        else if bottom_edge.is_some() && top_edge.is_none() {
            top_edge = Some(bottom_edge.unwrap() - self.base_size.height as i32);
        }

        let mut rect = Rect { position: parent_position, size: self.base_size };
        if left_edge.is_some() {
            rect.position.x = left_edge.unwrap();
            rect.size.width = u32::try_from(right_edge.unwrap() - left_edge.unwrap()).unwrap_or_default();
        }
        if top_edge.is_some() {
            rect.position.y = top_edge.unwrap();
            rect.size.height = u32::try_from(bottom_edge.unwrap() - top_edge.unwrap()).unwrap_or_default();
        }
        rect
    }
}
