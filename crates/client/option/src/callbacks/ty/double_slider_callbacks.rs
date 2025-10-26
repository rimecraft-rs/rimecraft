use math::MathDeltaExt as _;
use rimecraft_text::ProvideTextTy;

use crate::callbacks::{Callbacks, ty::SliderCallbacks};

/// A slider callback for [`f32`] values.
pub trait DoubleSliderCallbacks<Cx>: SliderCallbacks<f32, Cx>
where
    Cx: ProvideTextTy,
{
    /// Returns the minimum inclusive value for the slider.
    fn min_inclusive(&self) -> f32;

    /// Returns the maximum inclusive value for the slider.
    fn max_inclusive(&self) -> f32;

    /// Converts a value to slider progress (0.0 to 1.0).
    fn to_slider_progress(&self, value: f32) -> f32 {
        value
            .map(self.min_inclusive()..self.max_inclusive(), 0.0..1.0)
            .clamp(0.0, 1.0)
    }

    /// Converts slider progress (0.0 to 1.0) to a value.
    fn to_value(&self, slider_progress: f32) -> f32 {
        slider_progress
            .map(0.0..1.0, self.min_inclusive()..self.max_inclusive())
            .floor()
    }

    /// Validates the given `f32` value, returning `Some(validated_value)` if valid,
    /// or `None` if invalid.
    fn f32_validate(&self, value: f32) -> Option<f32>;

    /// Returns a new [`DoubleSliderCallbacks`] with the given modifier functions applied.
    fn with_modifier<R, IR, RI>(
        &self,
        progress_to_value: IR,
        value_to_progress: RI,
    ) -> impl SliderCallbacks<R, Cx>
    where
        IR: Fn(f32) -> Option<R> + Clone,
        RI: Fn(R) -> Option<f32>,
    {
        struct Impl<IR, RI, F, ToP, ToV> {
            progress_to_value: IR,
            value_to_progress: RI,
            f32_validate: F,
            to_slider_progress: ToP,
            to_value: ToV,
        }

        impl<R, IR, RI, F, ToP, ToV, Cx> SliderCallbacks<R, Cx> for Impl<IR, RI, F, ToP, ToV>
        where
            Cx: ProvideTextTy,
            IR: Fn(f32) -> Option<R> + Clone,
            RI: Fn(R) -> Option<f32>,
            F: Fn(f32) -> Option<f32> + Clone,
            ToP: Fn(f32) -> f32,
            ToV: Fn(f32) -> f32,
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
            IR: Fn(f32) -> Option<R> + Clone,
            RI: Fn(R) -> Option<f32>,
            F: Fn(f32) -> Option<f32> + Clone,
            ToP: Fn(f32) -> f32,
            ToV: Fn(f32) -> f32,
        {
            fn validate(&self, value: R) -> Option<R> {
                let i = (self.value_to_progress)(value);
                let validated = i.and_then(self.f32_validate.clone());
                validated.and_then(self.progress_to_value.clone())
            }
        }

        Impl {
            value_to_progress,
            progress_to_value,
            f32_validate: |value| self.f32_validate(value),
            to_slider_progress: |value| DoubleSliderCallbacks::to_slider_progress(self, value),
            to_value: |value| DoubleSliderCallbacks::to_value(self, value),
        }
    }
}
