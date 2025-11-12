//! Screen space operations.

use std::ops::{Add, Div, Mul, Sub};

use num_traits::{One, Zero};

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

impl<V> Zero for ScreenVec<V>
where
    V: Zero,
{
    fn zero() -> Self {
        Self {
            horizontal: V::zero(),
            vertical: V::zero(),
        }
    }

    fn is_zero(&self) -> bool {
        self.horizontal.is_zero() && self.vertical.is_zero()
    }
}

impl<V> One for ScreenVec<V>
where
    V: One + Copy,
{
    fn one() -> Self {
        Self {
            horizontal: V::one(),
            vertical: V::one(),
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

impl<V> Mul<V> for ScreenVec<V>
where
    V: Mul<Output = V> + Copy,
{
    type Output = Self;

    fn mul(self, rhs: V) -> Self::Output {
        Self::new(self.horizontal * rhs, self.vertical * rhs)
    }
}

impl<V> Mul<Self> for ScreenVec<V>
where
    V: Mul<Output = V> + Copy,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(
            self.horizontal * rhs.horizontal,
            self.vertical * rhs.vertical,
        )
    }
}

impl<V> Div<V> for ScreenVec<V>
where
    V: Div<Output = V> + Copy,
{
    type Output = Self;

    fn div(self, rhs: V) -> Self::Output {
        Self::new(self.horizontal / rhs, self.vertical / rhs)
    }
}

impl<V> Div<Self> for ScreenVec<V>
where
    V: Div<Output = V> + Copy,
{
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::new(
            self.horizontal / rhs.horizontal,
            self.vertical / rhs.vertical,
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
