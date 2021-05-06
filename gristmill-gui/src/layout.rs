use std::convert::TryFrom;

use gristmill::geometry2d::*;

use super::LayoutContext;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Side {
    Left, Right, Top, Bottom
}

impl Side {
    pub fn opposite(self) -> Side {
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AnchorTarget {
    None, Parent, PreviousSibling
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AnchorTargetSide {
    SameSide, OppositeSide, Center
}

#[derive(Copy, Clone, Debug)]
pub struct Anchor {
    pub target: AnchorTarget,
    pub target_side: AnchorTargetSide,
    pub offset: i32,
}

impl Default for Anchor {
    fn default() -> Anchor {
        Anchor { target: AnchorTarget::None, target_side: AnchorTargetSide::SameSide, offset: 0 }
    }
}

impl Anchor {
    pub fn none() -> Anchor { Anchor::default() }
    pub fn parent(offset: i32) -> Anchor {
        Anchor { target: AnchorTarget::Parent, target_side: AnchorTargetSide::SameSide, offset }
    }
    pub fn parent_opposite(offset: i32) -> Anchor {
        Anchor { target: AnchorTarget::Parent, target_side: AnchorTargetSide::OppositeSide, offset }
    }
    pub fn parent_center(offset: i32) -> Anchor {
        Anchor { target: AnchorTarget::Parent, target_side: AnchorTargetSide::Center, offset }
    }
    pub fn previous_sibling(offset: i32) -> Anchor {
        Anchor { target: AnchorTarget::PreviousSibling, target_side: AnchorTargetSide::SameSide, offset }
    }
    pub fn previous_sibling_opposite(offset: i32) -> Anchor {
        Anchor { target: AnchorTarget::PreviousSibling, target_side: AnchorTargetSide::OppositeSide, offset }
    }
}

#[derive(Default, Clone)]
pub struct Layout {
    pub base_size: Size,
    anchors: [Anchor; 4],
}

impl Layout {
    pub fn fill_parent(inset: i32) -> Layout {
        let anchor = Anchor::parent(inset);
        Layout {
            base_size: Size::zero(), anchors: [anchor; 4]
        }
    }
    pub fn center_parent(size: Size) -> Layout {
        let anchors = [
            Anchor::parent_center(-((size.width / 2) as i32)),
            Anchor::none(),
            Anchor::parent_center(-((size.height / 2) as i32)),
            Anchor::none(),
        ];
        Layout { base_size: size, anchors }
    }
    pub fn offset_parent(rect: Rect) -> Layout {
        let anchors = [
            Anchor::parent(rect.position.x),
            Anchor::none(),
            Anchor::parent(rect.position.y),
            Anchor::none(),
        ];
        Layout { base_size: rect.size, anchors }
    }
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
    fn get_edge(&self, context: &LayoutContext, side: Side) -> Option<i32> {
        let anchor = &self.anchors[side.to_index()];
        let target_rect = match anchor.target {
            AnchorTarget::None => return None,
            AnchorTarget::Parent => context.parent_rect(),
            AnchorTarget::PreviousSibling => context.previous_sibling_rect(),
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

    pub fn layout_self(&self, context: &LayoutContext) -> Rect {
        let mut left_edge = self.get_edge(context, Side::Left);
        let mut right_edge = self.get_edge(context, Side::Right);
        let mut top_edge = self.get_edge(context, Side::Top);
        let mut bottom_edge = self.get_edge(context, Side::Bottom);
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

        let mut rect = Rect { position: context.parent_rect().position, size: self.base_size };
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
