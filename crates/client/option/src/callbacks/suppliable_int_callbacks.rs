use std::fmt::Debug;

use rimecraft_text::ProvideTextTy;

use crate::callbacks::{
    Callbacks,
    ty::{
        CyclingCallbacks, IntSliderCallbacks, SliderCallbacks, TypeChangeableCallbacks,
        TypeChangeableMode,
    },
};

/// A set of callbacks for options with `i32` values and dynamic slider boundaries.
pub struct SuppliableIntCallbacks {
    /// A function that provides the minimum inclusive boundary for the slider.
    pub min_boundary: Box<dyn Fn() -> i32>,
    /// A function that provides the maximum inclusive boundary for the slider.
    pub max_boundary: Box<dyn Fn() -> i32>,
}

impl Debug for SuppliableIntCallbacks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SuppliableIntCallbacks")
            .field("min_boundary", &"<function>")
            .field("max_boundary", &"<function>")
            .finish()
    }
}

impl SuppliableIntCallbacks {
    /// Creates a new [`SuppliableIntCallbacks`] with the given boundary supplier functions.
    pub fn new<Min, Max>(min_boundary: Min, max_boundary: Max) -> Self
    where
        Min: Fn() -> i32 + 'static,
        Max: Fn() -> i32 + 'static,
    {
        Self {
            min_boundary: Box::new(min_boundary),
            max_boundary: Box::new(max_boundary),
        }
    }

    /// Creates a new [`SuppliableIntCallbacks`] with a fixed minimum boundary.
    pub fn with_fixed_min_boundary<F>(min: i32, max_boundary: F) -> Self
    where
        F: Fn() -> i32 + 'static,
    {
        Self {
            min_boundary: Box::new(move || min),
            max_boundary: Box::new(max_boundary),
        }
    }

    /// Creates a new [`SuppliableIntCallbacks`] with a fixed maximum boundary.
    pub fn with_fixed_max_boundary<F>(max: i32, min_boundary: F) -> Self
    where
        F: Fn() -> i32 + 'static,
    {
        Self {
            min_boundary: Box::new(min_boundary),
            max_boundary: Box::new(move || max),
        }
    }
}

impl<Cx> TypeChangeableCallbacks<i32, Cx> for SuppliableIntCallbacks
where
    Cx: ProvideTextTy,
{
    fn type_changeable_mode(&self) -> TypeChangeableMode {
        TypeChangeableMode::Cycling
    }
}

impl<Cx> CyclingCallbacks<i32, Cx> for SuppliableIntCallbacks
where
    Cx: ProvideTextTy,
{
    fn values(&self) {
        todo!()
    }
}

impl<Cx> IntSliderCallbacks<Cx> for SuppliableIntCallbacks
where
    Cx: ProvideTextTy,
{
    fn min_inclusive(&self) -> i32 {
        (self.min_boundary)()
    }

    fn max_inclusive(&self) -> i32 {
        (self.max_boundary)()
    }

    fn i32_validate(&self, value: i32) -> Option<i32> {
        value
            .clamp(
                <Self as IntSliderCallbacks<Cx>>::min_inclusive(self),
                <Self as IntSliderCallbacks<Cx>>::max_inclusive(self),
            )
            .into()
    }
}

impl<Cx> SliderCallbacks<i32, Cx> for SuppliableIntCallbacks
where
    Cx: ProvideTextTy,
{
    fn to_slider_progress(&self, value: i32) -> f32 {
        <Self as IntSliderCallbacks<Cx>>::to_slider_progress(self, value)
    }

    fn to_value(&self, slider_progress: f32) -> i32 {
        <Self as IntSliderCallbacks<Cx>>::to_value(self, slider_progress)
    }
}

impl<Cx> Callbacks<i32, Cx> for SuppliableIntCallbacks
where
    Cx: ProvideTextTy,
{
    fn validate(&self, value: i32) -> Option<i32> {
        <Self as IntSliderCallbacks<Cx>>::i32_validate(self, value)
    }
}
