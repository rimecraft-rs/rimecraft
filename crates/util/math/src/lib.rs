//! Math library for Rimecraft

use std::f32::NAN;

/// Returns the larger one of the two [`f32`].
pub fn max_f32(a: f32, b: f32) -> f32 {
    if a == NAN && b == NAN {
        NAN
    } else if a == NAN {
        b
    } else if b == NAN || a > b {
        a
    } else {
        b
    }
}

/// Returns the smaller one of the two [`f32`].
pub fn min_f32(a: f32, b: f32) -> f32 {
    if a == NAN && b == NAN {
        NAN
    } else if a == NAN {
        b
    } else if b == NAN || a < b {
        a
    } else {
        b
    }
}

/// Returns if a [`f32`] is in the specified range.
pub fn in_range(value: f32, min: f32, max: f32) -> bool {
    value >= min && value <= max
}

/// Clamps a [`f32`] to an inclusive range.
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    max_f32(min, min_f32(max, value))
}

/// Linear interpolates an [`f32`] between two values by a factor.
pub fn _lerp(factor: f32, start: f32, end: f32) -> f32 {
    start + (end - start) * factor
}

/// Linear interpolates an [`f32`] between two values by a factor, with argument `clamp` available to clamp the result.
pub fn lerp(factor: f32, start: f32, end: f32, clamps: bool) -> f32 {
    let lerp = _lerp(factor, start, end);
    if clamps {
        clamp(lerp, start, end)
    } else {
        lerp
    }
}

/// Gets the factor of a [`f32`] in a linear interpolation progress.
pub fn _get_lerp_factor(value: f32, start: f32, end: f32) -> f32 {
    (value - start) / (end - start)
}

/// Gets the factor of a [`f32`] in a linear interpolation progress, with argument `clamp` available to clamp the result.
pub fn get_lerp_factor(value: f32, start: f32, end: f32, clamps: bool) -> f32 {
    let factor = _get_lerp_factor(value, start, end);
    if clamps {
        clamp(factor, 0.0, 1.0)
    } else {
        factor
    }
}

/// Linearly maps a [`f32`] from an old range to a newer one.
pub fn _map(value: f32, old_start: f32, old_end: f32, new_start: f32, new_end: f32) -> f32 {
    _lerp(
        _get_lerp_factor(value, old_start, old_end),
        new_start,
        new_end,
    )
}

/// Linearly maps a [`f32`] from an old range to a newer one, with argument `clamps` available to clamp the result.
pub fn map(
    value: f32,
    old_start: f32,
    old_end: f32,
    new_start: f32,
    new_end: f32,
    clamps: bool,
) -> f32 {
    lerp(
        get_lerp_factor(value, old_start, old_end, clamps),
        new_start,
        new_end,
        clamps,
    )
}
