use rimecraft_math::MathDeltaExt;
use rimecraft_text::ProvideTextTy;

use crate::callbacks::{
    Callbacks,
    ty::{IntSliderCallbacks, SliderCallbacks},
};

/// A slider callback for validating `i32` values within a specified range.
///
/// See: [`IntSliderCallbacks`]
#[derive(Debug, Clone, Copy)]
pub struct ValidatingIntSliderCallbacks {
    min: i32,
    max: i32,
    /// Whether the slider applies values immediately upon change.
    pub applies_values_immediately: bool,
}

impl ValidatingIntSliderCallbacks {
    /// Creates a new [`ValidatingIntSliderCallbacks`] with the given minimum and maximum values.
    ///
    /// # Panics
    ///
    /// Panics if `min` is greater than `max`.
    pub fn new(min: i32, max: i32, applies_values_immediately: bool) -> Self {
        assert!(min <= max, "min ({}) must be <= max ({})", min, max);
        Self {
            min,
            max,
            applies_values_immediately,
        }
    }
}

impl<Cx> IntSliderCallbacks<Cx> for ValidatingIntSliderCallbacks
where
    Cx: ProvideTextTy,
{
    fn min_inclusive(&self) -> i32 {
        self.min
    }

    fn max_inclusive(&self) -> i32 {
        self.max
    }

    fn to_slider_progress(&self, value: i32) -> f32 {
        <f32 as MathDeltaExt<f32>>::map(
            value as f32 + 0.5,
            <Self as IntSliderCallbacks<Cx>>::min_inclusive(self) as f32
                ..<Self as IntSliderCallbacks<Cx>>::max_inclusive(self) as f32 + 1.0,
            0.0..1.0,
        )
        .clamp(0.0, 1.0)
    }

    fn to_value(&self, slider_progress: f32) -> i32 {
        <f32 as MathDeltaExt<f32>>::map(
            slider_progress,
            0.0..1.0,
            <Self as IntSliderCallbacks<Cx>>::min_inclusive(self) as f32
                ..<Self as IntSliderCallbacks<Cx>>::max_inclusive(self) as f32 + 1.0,
        )
        .floor() as i32
    }

    fn i32_validate(&self, value: i32) -> Option<i32> {
        Some(value).filter(|&value| value >= self.min && value <= self.max)
    }

    fn with_modifier<R, IR, RI>(
        &self,
        progress_to_value: IR,
        value_to_progress: RI,
    ) -> impl SliderCallbacks<R, Cx>
    where
        IR: Fn(i32) -> Option<R> + Clone,
        RI: Fn(R) -> Option<i32>,
    {
        struct Impl<IR, RI, F, ToP, ToV> {
            progress_to_value: IR,
            value_to_progress: RI,
            i32_validate: F,
            to_slider_progress: ToP,
            to_value: ToV,
        }

        impl<R, IR, RI, F, ToP, ToV, Cx> SliderCallbacks<R, Cx> for Impl<IR, RI, F, ToP, ToV>
        where
            Cx: ProvideTextTy,
            IR: Fn(i32) -> Option<R> + Clone,
            RI: Fn(R) -> Option<i32>,
            F: Fn(i32) -> Option<i32> + Clone,
            ToP: Fn(i32) -> f32,
            ToV: Fn(f32) -> i32,
        {
            fn to_slider_progress(&self, value: R) -> f32 {
                let progress = (self.value_to_progress)(value).unwrap();
                (self.to_slider_progress)(progress)
            }

            fn to_value(&self, slider_progress: f32) -> R {
                let value = (self.to_value)(slider_progress);
                (self.progress_to_value)(value).unwrap()
            }
        }

        impl<R, IR, RI, F, ToP, ToV, Cx> Callbacks<R, Cx> for Impl<IR, RI, F, ToP, ToV>
        where
            Cx: ProvideTextTy,
            IR: Fn(i32) -> Option<R> + Clone,
            RI: Fn(R) -> Option<i32>,
            F: Fn(i32) -> Option<i32> + Clone,
            ToP: Fn(i32) -> f32,
            ToV: Fn(f32) -> i32,
        {
            fn validate(&self, value: R) -> Option<R> {
                let i = (self.value_to_progress)(value);
                let validated = i.and_then(self.i32_validate.clone());
                validated.and_then(self.progress_to_value.clone())
            }
        }

        Impl {
            value_to_progress,
            progress_to_value,
            i32_validate: |value| <Self as IntSliderCallbacks<Cx>>::i32_validate(self, value),
            to_slider_progress: |value| {
                <Self as IntSliderCallbacks<Cx>>::to_slider_progress(self, value)
            },
            to_value: |value| <Self as IntSliderCallbacks<Cx>>::to_value(self, value),
        }
    }
}

impl<Cx> SliderCallbacks<i32, Cx> for ValidatingIntSliderCallbacks
where
    Cx: ProvideTextTy,
{
    fn applies_values_immediately(&self) -> bool {
        self.applies_values_immediately
    }

    fn to_slider_progress(&self, value: i32) -> f32 {
        <Self as IntSliderCallbacks<Cx>>::to_slider_progress(self, value)
    }

    fn to_value(&self, slider_progress: f32) -> i32 {
        <Self as IntSliderCallbacks<Cx>>::to_value(self, slider_progress)
    }
}

impl<Cx> Callbacks<i32, Cx> for ValidatingIntSliderCallbacks
where
    Cx: ProvideTextTy,
{
    fn validate(&self, value: i32) -> Option<i32> {
        <Self as IntSliderCallbacks<Cx>>::i32_validate(self, value)
    }
}
