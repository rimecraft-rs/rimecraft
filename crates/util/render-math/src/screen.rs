//! Screen space operations.

use std::ops::{Add, Sub};

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

    /// Returns `true` if both horizontal and vertical values are zero, i.e., equal to their default value.
    pub fn is_zero(&self) -> bool
    where
        V: PartialEq + Default,
    {
        self.horizontal == V::default() && self.vertical == V::default()
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
}

impl ScreenRect {
    /// Whether the position is at the origin (0, 0).
    pub fn is_at_origin(&self) -> bool {
        self.pos.is_zero()
    }

    /// Whether the size is zero (width and height are both 0).
    pub fn is_zero(&self) -> bool {
        self.size.is_zero()
    }
}
