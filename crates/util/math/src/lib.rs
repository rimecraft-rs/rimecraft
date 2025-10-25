//! Math library for Rimecraft.

use std::ops::Range;

use num_traits::{Float, Num, NumCast, Signed};

pub mod int;

/// Extension trait for common mathematical operations.
pub trait MathExt
where
    Self: Copy + Num,
{
    /// The type used as interpolation factors.
    type Factor: Sized + Copy;

    /// Converts `self` to a factor.
    fn to_factor(self) -> Self::Factor;

    /// Converts a factor back to the original type.
    fn from_factor(factor: Self::Factor) -> Self;

    /// Lerps between `self` and `to` by the given `factor`.
    fn lerp(self, to: Self, factor: Self::Factor) -> Self {
        self + (to - self) * Self::from_factor(factor)
    }

    /// Calculates the progress of `self` within the given `range`.
    fn progress(self, range: Range<Self>) -> Self::Factor {
        ((self - range.start) / (range.end - range.start)).to_factor()
    }

    /// Maps `self` from the `from` range to the `to` range, preserving the relative position.
    fn map(self, from: Range<Self>, to: Range<Self>) -> Self {
        to.start.lerp(to.end, self.progress(from))
    }
}

impl MathExt for f32 {
    type Factor = f32;

    fn to_factor(self) -> Self::Factor {
        self
    }

    fn from_factor(factor: Self::Factor) -> Self {
        factor
    }
}

impl MathExt for i32 {
    type Factor = f32;

    fn to_factor(self) -> Self::Factor {
        self as Self::Factor
    }

    fn from_factor(factor: Self::Factor) -> Self {
        factor as Self
    }
}

/// Finds the minimum value in the given range that satisfies the *monotonic predicate* `p`.
///
/// A *monotonic predicate* is one that is true after a specific point, and false before that point.
pub fn binary_search_ie_u32<F>(range: Range<u32>, p: F) -> Option<u32>
where
    F: Fn(u32) -> bool,
{
    let (mut min, max) = (range.start, range.end);
    let mut i = max - min;
    while i > 0 {
        let j = i / 2;
        let k = min + j;
        if p(k) {
            i = j;
        } else {
            min = k + 1;
            i -= j + 1;
        }
    }
    (min != max).then_some(min)
}

/// Linearly interpolates between `start` and `end` by the given `delta`.
#[allow(clippy::missing_panics_doc)]
#[inline]
pub fn lerp<T, Delta>(delta: Delta, start: T, end: T) -> T
where
    T: Signed + NumCast + Copy,
    Delta: Float,
{
    start + T::from((delta * Delta::from(end - start).unwrap()).floor()).unwrap()
}
