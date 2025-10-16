use std::fmt::Debug;

use rimecraft_text::ProvideTextTy;

use crate::callbacks::{
    Callbacks,
    ty::{CyclingCallbacks, IntSliderCallbacks, SliderCallbacks, TypeChangeableCallbacks},
};

pub struct SuppliableIntCallbacks {
    pub min_boundary: Box<dyn Fn() -> i32>,
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

    pub fn with_fixed_min_boundary<F>(min: i32, max_boundary: F) -> Self
    where
        F: Fn() -> i32 + 'static,
    {
        Self {
            min_boundary: Box::new(move || min),
            max_boundary: Box::new(max_boundary),
        }
    }

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

impl<Txt> TypeChangeableCallbacks<i32, Txt> for SuppliableIntCallbacks
where
    Txt: ProvideTextTy,
{
    fn is_cycling(&self) -> bool {
        true
    }
}

impl<Txt> CyclingCallbacks<i32, Txt> for SuppliableIntCallbacks
where
    Txt: ProvideTextTy,
{
    fn get_values(&self) {
        todo!()
    }
}

impl<Txt> IntSliderCallbacks<Txt> for SuppliableIntCallbacks
where
    Txt: ProvideTextTy,
{
    fn min_inclusive(&self) -> i32 {
        (self.min_boundary)()
    }

    fn max_inclusive(&self) -> i32 {
        (self.max_boundary)()
    }

    fn i32_validate(&self, value: Option<i32>) -> Option<i32> {
        value.map(|value| {
            value.clamp(
                <SuppliableIntCallbacks as IntSliderCallbacks<Txt>>::min_inclusive(self),
                <SuppliableIntCallbacks as IntSliderCallbacks<Txt>>::max_inclusive(self),
            )
        })
    }
}

impl<Txt> SliderCallbacks<i32, Txt> for SuppliableIntCallbacks
where
    Txt: ProvideTextTy,
{
    fn to_slider_progress(&self, value: i32) -> f32 {
        <SuppliableIntCallbacks as IntSliderCallbacks<Txt>>::to_slider_progress(self, value)
    }

    fn to_value(&self, slider_progress: f32) -> i32 {
        <SuppliableIntCallbacks as IntSliderCallbacks<Txt>>::to_value(self, slider_progress)
    }
}

impl<Txt> Callbacks<i32, Txt> for SuppliableIntCallbacks
where
    Txt: ProvideTextTy,
{
    fn validate(&self, value: Option<i32>) -> Option<i32> {
        <SuppliableIntCallbacks as IntSliderCallbacks<Txt>>::i32_validate(self, value)
    }
}
