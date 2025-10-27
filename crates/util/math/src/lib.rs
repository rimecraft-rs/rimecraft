//! Math library for Rimecraft.

use std::ops::Range;

use num_traits::{Float, NumCast, Signed};

pub mod int;

/// Extension trait for mathematical operations involving deltas and ranges.
pub trait MathDeltaExt
where
    Self: Copy + Signed + NumCast,
{
    /// The floating-point type used for delta calculations.
    type Delta: Float;

    /// Linearly interpolates between `self` and `to` by the given `delta` (0.0 to 1.0).
    fn lerp(self, to: Self, delta: Self::Delta) -> Self {
        self + Self::from((delta * <Self::Delta as NumCast>::from(to - self).unwrap()).floor())
            .unwrap()
    }

    /// Calculates the normalized delta of `self` within the given `range`.
    fn delta(self, range: Range<Self>) -> Self::Delta {
        if range.start == range.end {
            return NumCast::from(0.0).unwrap();
        }
        let delta = self - range.start;
        let span = range.end - range.start;
        <Self::Delta as NumCast>::from(delta).unwrap()
            / <Self::Delta as NumCast>::from(span).unwrap()
    }

    /// Maps `self` from the `from` range to the `to` range.
    fn map(self, from: Range<Self>, to: Range<Self>) -> Self {
        to.start.lerp(to.end, self.delta(from))
    }
}

impl MathDeltaExt for f32 {
    type Delta = f32;
}

impl MathDeltaExt for f64 {
    type Delta = f64;
}

impl MathDeltaExt for i8 {
    type Delta = f32;
}

impl MathDeltaExt for i16 {
    type Delta = f32;
}

impl MathDeltaExt for i32 {
    type Delta = f32;
}

impl MathDeltaExt for i64 {
    type Delta = f64;
}

impl MathDeltaExt for i128 {
    type Delta = f64;
}

impl MathDeltaExt for isize {
    type Delta = f64;
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

#[test]
fn test_math_delta_ext_integers() {
    assert_eq!(5i32.lerp(15, 0.0), 5);
    assert_eq!(5i32.lerp(15, 0.5), 10);
    assert_eq!(5i32.lerp(15, 1.0), 15);
    assert_eq!(5i32.lerp(15, -1.0), -5);
    assert_eq!(5i32.lerp(15, 2.0), 25);

    assert_eq!(5i32.delta(0..10), 0.5);
    assert_eq!(0i32.delta(0..0), 0.0);

    assert_eq!(5i32.map(0..10, 10..20), 15);
}

#[test]
fn test_math_delta_ext_floats() {
    assert_eq!(5.0f32.lerp(15.0, 0.0), 5.0);
    assert_eq!(5.0f32.lerp(15.0, 0.5), 10.0);
    assert_eq!(5.0f32.lerp(15.0, 1.0), 15.0);
    assert_eq!(5.0f32.lerp(15.0, -1.0), -5.0);
    assert_eq!(5.0f32.lerp(15.0, 2.0), 25.0);

    assert_eq!(5.0f32.delta(0.0..10.0), 0.5);
    assert_eq!(0.0f32.delta(0.0..0.0), 0.0);

    assert_eq!(5.0f32.map(0.0..10.0, 10.0..20.0), 15.0);
}

#[test]
fn test_binary_search_ie_u32() {
    let range = 0..100;
    let target = 73;
    let result = binary_search_ie_u32(range.clone(), |x| x >= target);
    assert_eq!(result, Some(target));

    let result_none = binary_search_ie_u32(range.clone(), |x| x >= 150);
    assert_eq!(result_none, None);

    let result_start = binary_search_ie_u32(range.clone(), |_| true);
    assert_eq!(result_start, Some(0));
}
