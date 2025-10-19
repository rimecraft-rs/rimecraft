//! Screen space operations and abstractions.

use std::ops::{Add, Sub};

use num_traits::One;

use crate::nav::{NavAxis, NavDirection, Sign};

/// A vector on the screen in 2D space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenVec<V> {
    /// The value on the horizontal axis.
    pub horizontal: V,
    /// The value on the vertical axis.
    pub vertical: V,
}

impl<V> Default for ScreenVec<V>
where
    V: Default,
{
    fn default() -> Self {
        Self {
            horizontal: V::default(),
            vertical: V::default(),
        }
    }
}

impl<V> ScreenVec<V> {
    /// Creates a new [`ScreenVec`] from the given horizontal and vertical values.
    pub const fn new(horizontal: V, vertical: V) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    /// Creates a new [`ScreenVec`] from the given axis, this axis value, and other axis value.
    pub const fn from_axis(axis: NavAxis, this_axis: V, other_axis: V) -> Self {
        match axis {
            NavAxis::Horizontal => Self::new(this_axis, other_axis),
            NavAxis::Vertical => Self::new(other_axis, this_axis),
        }
    }

    /// Returns `true` if both horizontal and vertical values are zero, i.e., equal to their default value.
    pub fn is_zero(&self) -> bool
    where
        V: PartialEq + Default,
    {
        self.horizontal == V::default() && self.vertical == V::default()
    }
}

impl<V> ScreenVec<V> {
    /// Returns the value on the given axis.
    pub fn get(&self, axis: NavAxis) -> &V {
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

impl<V> Add for ScreenVec<V>
where
    V: Add<Output = V>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(
            self.horizontal + rhs.horizontal,
            self.vertical + rhs.vertical,
        )
    }
}

impl<V> Sub for ScreenVec<V>
where
    V: Sub<Output = V>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(
            self.horizontal - rhs.horizontal,
            self.vertical - rhs.vertical,
        )
    }
}

impl From<glam::Vec2> for ScreenVec<f32> {
    fn from(vec: glam::Vec2) -> Self {
        Self::new(vec.x, vec.y)
    }
}

/// A position on the screen in 2D space.
pub type ScreenPos = ScreenVec<f32>;

/// A size on the screen in 2D space.
pub type ScreenSize = ScreenVec<f32>;

/// A rectangle on the screen in 2D space.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ScreenRect {
    /// The position of the rectangle.
    pub pos: ScreenPos,
    /// The size of the rectangle.
    pub size: ScreenSize,
}

impl ScreenRect {
    /// Creates a new [`ScreenRect`] from the given position and size.
    pub const fn new(pos: ScreenPos, size: ScreenSize) -> Self {
        Self { pos, size }
    }

    /// Creates a new [`ScreenRect`] from the given x, y, width and height.
    pub const fn from_xywh(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            pos: ScreenPos::new(x, y),
            size: ScreenSize::new(w, h),
        }
    }

    /// Creates a new [`ScreenRect`] from the given axis, this axis position, other axis position,
    /// this axis size, and other axis size.
    pub const fn from_axis(
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
}

impl ScreenRect {
    pub fn is_at_origin(&self) -> bool {
        self.pos.is_zero()
    }

    pub fn is_zero(&self) -> bool {
        self.size.is_zero()
    }

    pub fn length(&self, axis: NavAxis) -> f32 {
        match axis {
            NavAxis::Horizontal => self.size.horizontal,
            NavAxis::Vertical => self.size.vertical,
        }
    }

    pub fn coord(&self, direction: NavDirection) -> f32 {
        match direction.sign() {
            Sign::Positive => self.pos.get(direction.axis()) + self.length(direction.axis()),
            Sign::Negative => self.pos.get(direction.axis()).to_owned(),
        }
    }

    pub fn border(&self, direction: NavDirection) -> ScreenRect {
        let opposite_axis = direction.axis().flip();
        let coord_start = self.coord(direction);
        let coord_end = self.coord(opposite_axis.direction(Sign::Negative));
        let length = self.length(opposite_axis);
        ScreenRect::from_axis(direction.axis(), coord_start, coord_end, 1.0, length) + direction
    }

    pub fn overlaps_axis(&self, other: &Self, axis: NavAxis) -> bool {
        let self_start = self.coord(axis.direction(Sign::Negative));
        let self_end = self.coord(axis.direction(Sign::Positive));
        let other_start = other.coord(axis.direction(Sign::Negative));
        let other_end = other.coord(axis.direction(Sign::Positive));
        self_start.max(other_start) <= self_end.min(other_end)
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        self.overlaps_axis(other, NavAxis::Horizontal)
            && self.overlaps_axis(other, NavAxis::Vertical)
    }

    pub fn center_axis(&self, axis: NavAxis) -> f32 {
        let start = self.coord(axis.direction(Sign::Negative));
        let end = self.coord(axis.direction(Sign::Positive));
        (start + end) / 2.0
    }

    pub fn center(&self) -> ScreenPos {
        ScreenPos::new(
            self.center_axis(NavAxis::Horizontal),
            self.center_axis(NavAxis::Vertical),
        )
    }

    pub fn coord_top(&self) -> f32 {
        self.coord(NavDirection::Up)
    }

    pub fn coord_bottom(&self) -> f32 {
        self.coord(NavDirection::Down)
    }

    pub fn coord_left(&self) -> f32 {
        self.coord(NavDirection::Left)
    }

    pub fn coord_right(&self) -> f32 {
        self.coord(NavDirection::Right)
    }

    pub fn intersection(&self, other: &Self) -> Option<Self> {
        if self.is_zero() || other.is_zero() {
            return None;
        }

        let x1 = self.coord_left().max(other.coord_left());
        let y1 = self.coord_top().max(other.coord_top());
        let x2 = self.coord_right().min(other.coord_right());
        let y2 = self.coord_bottom().min(other.coord_bottom());

        if x2 >= x1 && y2 >= y1 {
            Some(ScreenRect::from_xywh(x1, y1, x2 - x1, y2 - y1))
        } else {
            None
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.intersection(other).is_some()
    }

    pub fn union(&self, other: &Self) -> Self {
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

        ScreenRect::from_xywh(x1, y1, x2 - x1, y2 - y1)
    }

    pub fn contains(&self, other: &Self) -> bool {
        self.coord_left() <= other.coord_left()
            && self.coord_right() >= other.coord_right()
            && self.coord_top() <= other.coord_top()
            && self.coord_bottom() >= other.coord_bottom()
    }

    pub fn contains_pos(&self, pos: ScreenPos) -> bool {
        let x = pos.horizontal;
        let y = pos.vertical;
        x >= self.coord_left()
            && x <= self.coord_right()
            && y >= self.coord_top()
            && y <= self.coord_bottom()
    }

    pub fn transform(&self, transformation: glam::Affine2) -> Self {
        let top_left =
            transformation.transform_point2(glam::Vec2::new(self.coord_left(), self.coord_top()));
        let bottom_right = transformation
            .transform_point2(glam::Vec2::new(self.coord_right(), self.coord_bottom()));
        ScreenRect::from_xywh(
            top_left.x,
            top_left.y,
            bottom_right.x - top_left.x,
            bottom_right.y - top_left.y,
        )
    }

    pub fn transform_vertices(&self, transformation: glam::Affine2) -> ScreenRect {
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

        ScreenRect::from_xywh(min_x, min_y, max_x - min_x, max_y - min_y)
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
