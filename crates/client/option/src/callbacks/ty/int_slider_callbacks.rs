use rimecraft_math::MathDeltaExt;
use rimecraft_text::ProvideTextTy;

use crate::callbacks::{Callbacks, ty::SliderCallbacks};

/// A slider callback for [`i32`] values.
pub trait IntSliderCallbacks<Cx>: SliderCallbacks<i32, Cx>
where
    Cx: ProvideTextTy,
{
    /// Returns the minimum inclusive value for the slider.
    fn min_inclusive(&self) -> i32;

    /// Returns the maximum inclusive value for the slider.
    fn max_inclusive(&self) -> i32;

    /// Converts a value to slider progress (0.0 to 1.0).
    fn to_slider_progress(&self, value: i32) -> f32 {
        <f32 as MathDeltaExt<f32>>::map(
            value as f32 + 0.5,
            self.min_inclusive() as f32..self.max_inclusive() as f32 + 1.0,
            0.0..1.0,
        )
        .clamp(0.0, 1.0)
    }

    /// Converts slider progress (0.0 to 1.0) to a value.
    fn to_value(&self, slider_progress: f32) -> i32 {
        <f32 as MathDeltaExt<f32>>::map(
            slider_progress,
            0.0..1.0,
            self.min_inclusive() as f32..self.max_inclusive() as f32 + 1.0,
        )
        .floor() as i32
    }

    /// Validates the given `i32` value, returning `Some(validated_value)` if valid,
    fn i32_validate(&self, value: i32) -> Option<i32>;

    /// Returns a new [`IntSliderCallbacks`] with the given modifier functions applied.
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
            i32_validate: |value| self.i32_validate(value),
            to_slider_progress: |value| IntSliderCallbacks::to_slider_progress(self, value),
            to_value: |value| IntSliderCallbacks::to_value(self, value),
        }
    }
}
