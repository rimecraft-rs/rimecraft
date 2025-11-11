//! Navigation operations in screen space.

use std::ops::{Add, Sub};

use super::*;

use num_traits::One;
use rimecraft_render_math::screen::{ScreenPos, ScreenRect, ScreenVec};

/// Extension methods for [`ScreenVec`] to support navigation operations.
pub trait ScreenVecExt<V> {
    /// Creates a new [`ScreenVec`] from the given axis, this axis value, and other axis value.
    fn from_axis(axis: NavAxis, this_axis: V, other_axis: V) -> Self;

    /// Returns the value on the given axis.
    fn get(&self, axis: NavAxis) -> &V;
}

impl<V> ScreenVecExt<V> for ScreenVec<V> {
    fn from_axis(axis: NavAxis, this_axis: V, other_axis: V) -> Self {
        match axis {
            NavAxis::Horizontal => Self::new(this_axis, other_axis),
            NavAxis::Vertical => Self::new(other_axis, this_axis),
        }
    }

    fn get(&self, axis: NavAxis) -> &V {
        match axis {
            NavAxis::Horizontal => &self.horizontal,
            NavAxis::Vertical => &self.vertical,
        }
    }
}

impl<V> Add<NavDirection> for ScreenVec<V>
where
    V: Add<Output = V> + Sub<Output = V> + One,
{
    type Output = Self;

    /// Moves the value in the given navigation direction by `1.0` unit.
    fn add(self, rhs: NavDirection) -> Self::Output {
        match rhs {
            NavDirection::Up => Self::new(self.horizontal, self.vertical - V::one()),
            NavDirection::Down => Self::new(self.horizontal, self.vertical + V::one()),
            NavDirection::Left => Self::new(self.horizontal - V::one(), self.vertical),
            NavDirection::Right => Self::new(self.horizontal + V::one(), self.vertical),
        }
    }
}

impl<V> Sub<NavDirection> for ScreenVec<V>
where
    V: Add<Output = V> + Sub<Output = V> + One,
{
    type Output = Self;

    /// Moves the value in the opposite of the given navigation direction by `1.0` unit.
    fn sub(self, rhs: NavDirection) -> Self::Output {
        self + (!rhs)
    }
}

/// Extension methods for [`ScreenRect`] to support navigation operations.
pub trait ScreenRectExt {
    /// Creates a new [`ScreenRect`] from the given axis, this axis position, other axis position,
    /// this axis size, and other axis size.
    fn from_axis(
        axis: NavAxis,
        this_axis_pos: f32,
        other_axis_pos: f32,
        this_axis_size: f32,
        other_axis_size: f32,
    ) -> Self;

    /// Returns the length along the given axis.
    fn length(&self, axis: NavAxis) -> f32;

    /// Returns the coordinate in the given direction.
    fn coord(&self, direction: NavDirection) -> f32;

    /// Returns the border rectangle in the given direction.
    fn border(&self, direction: NavDirection) -> Self;

    /// Checks if overlaps on the given axis.
    fn overlaps_axis(&self, other: &Self, axis: NavAxis) -> bool;

    /// Checks if overlaps on both axes.
    fn overlaps(&self, other: &Self) -> bool;

    /// Returns the center coordinate along the given axis.
    fn center_axis(&self, axis: NavAxis) -> f32;

    /// Returns the center position.
    fn center(&self) -> ScreenPos;

    /// Returns the top coordinate.
    fn coord_top(&self) -> f32;

    /// Returns the bottom coordinate.
    fn coord_bottom(&self) -> f32;

    /// Returns the left coordinate.
    fn coord_left(&self) -> f32;

    /// Returns the right coordinate.
    fn coord_right(&self) -> f32;

    /// Returns the intersection with another rectangle.
    fn intersection(&self, other: &Self) -> Option<Self>
    where
        Self: Sized;

    /// Checks if there is an intersection.
    fn intersects(&self, other: &Self) -> bool;

    /// Returns the union with another rectangle.
    fn union(&self, other: &Self) -> Self;

    /// Checks if contains another rectangle.
    fn contains(&self, other: &Self) -> bool;

    /// Checks if contains the given point.
    fn contains_pos(&self, pos: ScreenPos) -> bool;

    /// Applies an affine transformation to the rectangle.
    fn transform(&self, transformation: glam::Affine2) -> Self;

    /// Applies an affine transformation to vertices and returns the bounding box.
    fn transform_vertices(&self, transformation: glam::Affine2) -> Self;
}

impl ScreenRectExt for ScreenRect {
    fn from_axis(
        axis: NavAxis,
        this_axis_pos: f32,
        other_axis_pos: f32,
        this_axis_size: f32,
        other_axis_size: f32,
    ) -> Self {
        Self {
            pos: ScreenVec::from_axis(axis, this_axis_pos, other_axis_pos),
            size: ScreenVec::from_axis(axis, this_axis_size, other_axis_size),
        }
    }

    fn length(&self, axis: NavAxis) -> f32 {
        match axis {
            NavAxis::Horizontal => self.size.horizontal,
            NavAxis::Vertical => self.size.vertical,
        }
    }

    fn coord(&self, direction: NavDirection) -> f32 {
        match direction.sign() {
            Sign::Positive => self.pos.get(direction.axis()) + self.length(direction.axis()),
            Sign::Negative => self.pos.get(direction.axis()).to_owned(),
        }
    }

    fn border(&self, direction: NavDirection) -> Self {
        let opposite_axis = direction.axis().flip();
        let coord_start = self.coord(direction);
        let coord_end = self.coord(opposite_axis.direction(Sign::Negative));
        let length = self.length(opposite_axis);
        Self::from_axis(direction.axis(), coord_start, coord_end, 1.0, length) + direction
    }

    fn overlaps_axis(&self, other: &Self, axis: NavAxis) -> bool {
        let self_start = self.coord(axis.direction(Sign::Negative));
        let self_end = self.coord(axis.direction(Sign::Positive));
        let other_start = other.coord(axis.direction(Sign::Negative));
        let other_end = other.coord(axis.direction(Sign::Positive));
        self_start.max(other_start) <= self_end.min(other_end)
    }

    fn overlaps(&self, other: &Self) -> bool {
        self.overlaps_axis(other, NavAxis::Horizontal)
            && self.overlaps_axis(other, NavAxis::Vertical)
    }

    fn center_axis(&self, axis: NavAxis) -> f32 {
        let start = self.coord(axis.direction(Sign::Negative));
        let end = self.coord(axis.direction(Sign::Positive));
        f32::midpoint(start, end)
    }

    fn center(&self) -> ScreenPos {
        ScreenPos::new(
            self.center_axis(NavAxis::Horizontal),
            self.center_axis(NavAxis::Vertical),
        )
    }

    fn coord_top(&self) -> f32 {
        self.coord(NavDirection::Up)
    }

    fn coord_bottom(&self) -> f32 {
        self.coord(NavDirection::Down)
    }

    fn coord_left(&self) -> f32 {
        self.coord(NavDirection::Left)
    }

    fn coord_right(&self) -> f32 {
        self.coord(NavDirection::Right)
    }

    fn intersection(&self, other: &Self) -> Option<Self> {
        if self.is_zero() || other.is_zero() {
            return None;
        }

        let x1 = self.coord_left().max(other.coord_left());
        let y1 = self.coord_top().max(other.coord_top());
        let x2 = self.coord_right().min(other.coord_right());
        let y2 = self.coord_bottom().min(other.coord_bottom());

        (x2 >= x1 && y2 >= y1).then(|| Self::from_xywh(x1, y1, x2 - x1, y2 - y1))
    }

    fn intersects(&self, other: &Self) -> bool {
        self.intersection(other).is_some()
    }

    fn union(&self, other: &Self) -> Self {
        if self.is_zero() {
            return *other;
        }
        if other.is_zero() {
            return *self;
        }

        let x1 = self.coord_left().min(other.coord_left());
        let y1 = self.coord_top().min(other.coord_top());
        let x2 = self.coord_right().max(other.coord_right());
        let y2 = self.coord_bottom().max(other.coord_bottom());

        Self::from_xywh(x1, y1, x2 - x1, y2 - y1)
    }

    fn contains(&self, other: &Self) -> bool {
        self.coord_left() <= other.coord_left()
            && self.coord_right() >= other.coord_right()
            && self.coord_top() <= other.coord_top()
            && self.coord_bottom() >= other.coord_bottom()
    }

    fn contains_pos(&self, pos: ScreenPos) -> bool {
        let x = pos.horizontal;
        let y = pos.vertical;
        x >= self.coord_left()
            && x <= self.coord_right()
            && y >= self.coord_top()
            && y <= self.coord_bottom()
    }

    fn transform(&self, transformation: glam::Affine2) -> Self {
        let top_left =
            transformation.transform_point2(glam::Vec2::new(self.coord_left(), self.coord_top()));
        let bottom_right = transformation
            .transform_point2(glam::Vec2::new(self.coord_right(), self.coord_bottom()));
        Self::from_xywh(
            top_left.x,
            top_left.y,
            bottom_right.x - top_left.x,
            bottom_right.y - top_left.y,
        )
    }

    fn transform_vertices(&self, transformation: glam::Affine2) -> Self {
        let top_left =
            transformation.transform_point2(glam::Vec2::new(self.coord_left(), self.coord_top()));
        let top_right =
            transformation.transform_point2(glam::Vec2::new(self.coord_right(), self.coord_top()));
        let bottom_left = transformation
            .transform_point2(glam::Vec2::new(self.coord_left(), self.coord_bottom()));
        let bottom_right = transformation
            .transform_point2(glam::Vec2::new(self.coord_right(), self.coord_bottom()));

        let min_x = top_left
            .x
            .min(top_right.x)
            .min(bottom_left.x)
            .min(bottom_right.x);
        let max_x = top_left
            .x
            .max(top_right.x)
            .max(bottom_left.x)
            .max(bottom_right.x);
        let min_y = top_left
            .y
            .min(top_right.y)
            .min(bottom_left.y)
            .min(bottom_right.y);
        let max_y = top_left
            .y
            .max(top_right.y)
            .max(bottom_left.y)
            .max(bottom_right.y);

        Self::from_xywh(min_x, min_y, max_x - min_x, max_y - min_y)
    }
}

impl Add<NavDirection> for ScreenRect {
    type Output = Self;

    /// Moves the rectangle in the given navigation direction by `1.0` unit.
    fn add(self, rhs: NavDirection) -> Self::Output {
        Self {
            pos: self.pos + rhs,
            ..self
        }
    }
}

impl Sub<NavDirection> for ScreenRect {
    type Output = Self;

    /// Moves the rectangle in the opposite of the given navigation direction by `1.0` unit.
    fn sub(self, rhs: NavDirection) -> Self::Output {
        Self {
            pos: self.pos - rhs,
            ..self
        }
    }
}
