//! Math library for Rimecraft

use std::ops::Range;

/// Linear interpolates an [`f32`] between two values by a factor.
pub fn __lerp_f32(factor: f32, start: f32, end: f32) -> f32 {
    start + (end - start) * factor
}

/// Linear interpolates an [`f32`] between two values by a factor, with argument `clamp` available to clamp the result.
pub fn lerp_f32(factor: f32, start: f32, end: f32, clamps: bool) -> f32 {
    let lerp = __lerp_f32(factor, start, end);
    if clamps { lerp.clamp(start, end) } else { lerp }
}

/// Gets the factor of a [`f32`] in a linear interpolation progress.
pub fn __lerp_factor_f32(value: f32, start: f32, end: f32) -> f32 {
    (value - start) / (end - start)
}

/// Gets the factor of a [`f32`] in a linear interpolation progress, with argument `clamp` available to clamp the result.
pub fn lerp_factor_f32(value: f32, start: f32, end: f32, clamps: bool) -> f32 {
    let factor = __lerp_factor_f32(value, start, end);
    if clamps {
        factor.clamp(0.0, 1.0)
    } else {
        factor
    }
}

/// Linearly maps a [`f32`] from an old range to a newer one.
pub fn __map_f32(value: f32, old_start: f32, old_end: f32, new_start: f32, new_end: f32) -> f32 {
    __lerp_f32(
        __lerp_factor_f32(value, old_start, old_end),
        new_start,
        new_end,
    )
}

/// Linearly maps a [`f32`] from an old range to a newer one, with argument `clamps` available to clamp the result.
pub fn map_f32(
    value: f32,
    old_start: f32,
    old_end: f32,
    new_start: f32,
    new_end: f32,
    clamps: bool,
) -> f32 {
    lerp_f32(
        lerp_factor_f32(value, old_start, old_end, clamps),
        new_start,
        new_end,
        clamps,
    )
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
