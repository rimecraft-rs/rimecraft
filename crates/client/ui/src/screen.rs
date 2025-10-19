//! Screen space operations and abstractions.

use std::ops::{Add, Sub};

use num_traits::One;

use crate::nav::{NavAxis, NavDirection};

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
    is_empty: bool,
}

impl ScreenRect {
    /// Creates a new [`ScreenRect`] from the given position and size.
    pub const fn new(pos: ScreenPos, size: ScreenSize) -> Self {
        Self {
            pos,
            size,
            is_empty: false,
        }
    }

    /// Creates a new [`ScreenRect`] from the given x, y, width and height.
    pub const fn from_xywh(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            pos: ScreenPos::new(x, y),
            size: ScreenSize::new(w, h),
            is_empty: false,
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
            is_empty: false,
        }
    }

    /// Creates an empty [`ScreenRect`].
    pub fn empty() -> Self {
        Self {
            is_empty: true,
            ..Default::default()
        }
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
