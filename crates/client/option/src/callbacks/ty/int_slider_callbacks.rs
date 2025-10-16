use rimecraft_math::MathExt as _;
use rimecraft_text::ProvideTextTy;

use crate::callbacks::{Callbacks, ty::SliderCallbacks};

pub trait IntSliderCallbacks<Txt>: SliderCallbacks<i32, Txt>
where
    Txt: ProvideTextTy,
{
    fn min_inclusive(&self) -> i32;

    fn max_inclusive(&self) -> i32;

    fn to_slider_progress(&self, value: i32) -> f32 {
        (value as f32 + 0.5)
            .map(
                self.min_inclusive() as f32..self.max_inclusive() as f32 + 1.0,
                0.0..1.0,
            )
            .clamp(0.0, 1.0)
    }

    fn to_value(&self, slider_progress: f32) -> i32 {
        slider_progress
            .map(
                0.0..1.0,
                self.min_inclusive() as f32..self.max_inclusive() as f32 + 1.0,
            )
            .floor() as i32
    }

    fn i32_validate(&self, value: Option<i32>) -> Option<i32>;

    fn with_modifier<R, IR, RI, F, ToP, ToV>(
        &self,
        progress_to_value: IR,
        value_to_progress: RI,
    ) -> impl SliderCallbacks<R, Txt>
    where
        IR: Fn(Option<i32>) -> Option<R>,
        RI: Fn(Option<R>) -> Option<i32>,
        F: Fn(Option<i32>) -> Option<i32>,
        ToP: Fn(i32) -> f32,
        ToV: Fn(f32) -> i32,
    {
        struct Impl<IR, RI, F, ToP, ToV> {
            progress_to_value: IR,
            value_to_progress: RI,
            i32_validate: F,
            to_slider_progress: ToP,
            to_value: ToV,
        }

        impl<R, IR, RI, F, ToP, ToV, Txt> SliderCallbacks<R, Txt> for Impl<IR, RI, F, ToP, ToV>
        where
            Txt: ProvideTextTy,
            IR: Fn(Option<i32>) -> Option<R>,
            RI: Fn(Option<R>) -> Option<i32>,
            F: Fn(Option<i32>) -> Option<i32>,
            ToP: Fn(i32) -> f32,
            ToV: Fn(f32) -> i32,
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

        impl<R, IR, RI, F, ToP, ToV, Txt> Callbacks<R, Txt> for Impl<IR, RI, F, ToP, ToV>
        where
            Txt: ProvideTextTy,
            IR: Fn(Option<i32>) -> Option<R>,
            RI: Fn(Option<R>) -> Option<i32>,
            F: Fn(Option<i32>) -> Option<i32>,
            ToP: Fn(i32) -> f32,
            ToV: Fn(f32) -> i32,
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
            i32_validate: |value| self.i32_validate(value),
            to_slider_progress: |value| IntSliderCallbacks::to_slider_progress(self, value),
            to_value: |value| IntSliderCallbacks::to_value(self, value),
        }
    }
}
