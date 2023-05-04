pub mod navigation;

use self::navigation::{NavigationAxis, NavigationDirection};
use super::util::math::MatrixStack;
use std::{cmp, ops::Add};

pub struct DrawContext {
    matrices: MatrixStack,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ScreenRect {
    pub pos: ScreenPos,
    pub width: u32,
    pub height: u32,
}

impl ScreenRect {
    pub fn new(same_axis: i32, other_axis: i32, width: u32, height: u32) -> Self {
        Self {
            pos: ScreenPos(same_axis, other_axis),
            width,
            height,
        }
    }

    pub fn of(
        axis: NavigationAxis,
        same_axis_coord: i32,
        other_axis_coord: i32,
        same_axis_len: u32,
        other_axis_len: u32,
    ) -> Self {
        match axis {
            NavigationAxis::Horizontal => Self::new(
                same_axis_coord,
                other_axis_coord,
                same_axis_len,
                other_axis_len,
            ),
            NavigationAxis::Vertical => Self::new(
                other_axis_coord,
                same_axis_coord,
                other_axis_len,
                same_axis_len,
            ),
        }
    }

    pub fn len(&self, axis: NavigationAxis) -> u32 {
        match axis {
            NavigationAxis::Horizontal => self.width,
            NavigationAxis::Vertical => self.height,
        }
    }

    pub fn bounding_coord(&self, direction: NavigationDirection) -> i32 {
        let axis = direction.axis();
        if direction.is_positive() {
            self.pos.component(axis) + self.len(axis) as i32 - 1
        } else {
            self.pos.component(axis)
        }
    }

    pub fn border(&self, direction: NavigationDirection) -> Self {
        let i = self.bounding_coord(direction);
        let axis = direction.axis().other();
        let j = self.bounding_coord(axis.negative_direction());
        let k = self.len(axis);
        Self::of(direction.axis(), i, j, 1, k) + direction
    }

    pub fn overlaps(&self, other: Self, axis: Option<NavigationAxis>) -> bool {
        if let Some(axis_r) = axis {
            cmp::max(
                self.bounding_coord(axis_r.negative_direction()),
                other.bounding_coord(axis_r.negative_direction()),
            ) <= cmp::min(
                self.bounding_coord(axis_r.positive_direction()),
                other.bounding_coord(axis_r.positive_direction()),
            )
        } else {
            self.overlaps(other, Some(NavigationAxis::Horizontal))
                && self.overlaps(other, Some(NavigationAxis::Vertical))
        }
    }

    pub fn center(&self, axis: NavigationAxis) -> i32 {
        self.bounding_coord(axis.positive_direction())
            + self.bounding_coord(axis.negative_direction()) / 2
    }

    pub fn intersection(&self, other: Self) -> Option<Self> {
        let i = cmp::max(self.left(), other.left());
        let j = cmp::max(self.top(), other.top());
        let k = cmp::min(self.right(), other.right());
        let l = cmp::min(self.bottom(), other.bottom());
        if i >= k || j >= l {
            None
        } else {
            Some(Self::new(i, j, (k - 1) as u32, (l - j) as u32))
        }
    }

    pub fn top(&self) -> i32 {
        self.pos.1
    }

    pub fn bottom(&self) -> i32 {
        self.pos.1 + self.height as i32
    }

    pub fn left(&self) -> i32 {
        self.pos.0
    }

    pub fn right(&self) -> i32 {
        self.pos.0 + self.width as i32
    }
}

impl Add<NavigationDirection> for ScreenRect {
    type Output = ScreenRect;

    fn add(self, rhs: NavigationDirection) -> Self::Output {
        Self {
            pos: self.pos + rhs,
            width: self.width,
            height: self.height,
        }
    }
}

impl Default for ScreenRect {
    fn default() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ScreenPos(i32, i32);

impl ScreenPos {
    pub fn of(axis: NavigationAxis, same_axis: i32, other_axis: i32) -> Self {
        match axis {
            NavigationAxis::Horizontal => Self(same_axis, other_axis),
            NavigationAxis::Vertical => Self(other_axis, same_axis),
        }
    }

    pub fn component(&self, axis: NavigationAxis) -> i32 {
        match axis {
            NavigationAxis::Horizontal => self.0,
            NavigationAxis::Vertical => self.1,
        }
    }
}

impl Add<NavigationDirection> for ScreenPos {
    type Output = ScreenPos;

    fn add(self, rhs: NavigationDirection) -> Self::Output {
        match rhs {
            NavigationDirection::Up => Self(self.0, self.1 - 1),
            NavigationDirection::Down => Self(self.0, self.1 + 1),
            NavigationDirection::Left => Self(self.0 - 1, self.1),
            NavigationDirection::Right => Self(self.0 + 1, self.1),
        }
    }
}
