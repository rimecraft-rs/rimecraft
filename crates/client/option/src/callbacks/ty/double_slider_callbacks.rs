use rimecraft_math::MathExt as _;
use rimecraft_text::ProvideTextTy;

use crate::callbacks::{Callbacks, ty::SliderCallbacks};

pub trait DoubleSliderCallbacks<Cx>: SliderCallbacks<f32, Cx>
where
    Cx: ProvideTextTy,
{
    fn min_inclusive(&self) -> f32;

    fn max_inclusive(&self) -> f32;

    fn to_slider_progress(&self, value: f32) -> f32 {
        value
            .map(self.min_inclusive()..self.max_inclusive(), 0.0..1.0)
            .clamp(0.0, 1.0)
    }

    fn to_value(&self, slider_progress: f32) -> f32 {
        slider_progress
            .map(0.0..1.0, self.min_inclusive()..self.max_inclusive())
            .floor()
    }

    fn f32_validate(&self, value: Option<f32>) -> Option<f32>;

    fn with_modifier<R, IR, RI, F, ToP, ToV>(
        &self,
        progress_to_value: IR,
        value_to_progress: RI,
    ) -> impl SliderCallbacks<R, Cx>
    where
        IR: Fn(Option<f32>) -> Option<R>,
        RI: Fn(Option<R>) -> Option<f32>,
        F: Fn(Option<f32>) -> Option<f32>,
        ToP: Fn(f32) -> f32,
        ToV: Fn(f32) -> f32,
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
            IR: Fn(Option<f32>) -> Option<R>,
            RI: Fn(Option<R>) -> Option<f32>,
            F: Fn(Option<f32>) -> Option<f32>,
            ToP: Fn(f32) -> f32,
            ToV: Fn(f32) -> f32,
        {
            fn to_slider_progress(&self, value: R) -> f32 {
                let progress = (self.value_to_progress)(Some(value)).unwrap();
                (self.to_slider_progress)(progress)
            }

            fn to_value(&self, slider_progress: f32) -> R {
                let value = (self.to_value)(slider_progress);
                (self.progress_to_value)(Some(value)).unwrap()
            }
        }

        impl<R, IR, RI, F, ToP, ToV, Cx> Callbacks<R, Cx> for Impl<IR, RI, F, ToP, ToV>
        where
            Cx: ProvideTextTy,
            IR: Fn(Option<f32>) -> Option<R>,
            RI: Fn(Option<R>) -> Option<f32>,
            F: Fn(Option<f32>) -> Option<f32>,
            ToP: Fn(f32) -> f32,
            ToV: Fn(f32) -> f32,
        {
            fn validate(&self, value: Option<R>) -> Option<R> {
                let i = (self.value_to_progress)(value);
                let invalidated = (self.i32_validate)(i);
                (self.progress_to_value)(invalidated)
            }
        }

        Impl {
            value_to_progress,
            progress_to_value,
            i32_validate: |value| self.f32_validate(value),
            to_slider_progress: |value| DoubleSliderCallbacks::to_slider_progress(self, value),
            to_value: |value| DoubleSliderCallbacks::to_value(self, value),
        }
    }
}
