use rimecraft_math::MathExt as _;
use rimecraft_text::ProvideTextTy;

use crate::callbacks::{
    Callbacks,
    ty::{IntSliderCallbacks, SliderCallbacks},
};

#[derive(Debug, Clone, Copy)]
pub struct ValidatingIntSliderCallbacks {
    min: i32,
    max: i32,
    pub applies_values_immediately: bool,
}

impl ValidatingIntSliderCallbacks {
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

    fn applies_values_immediately(&self) -> bool {
        self.applies_values_immediately
    }

    fn to_slider_progress(&self, value: i32) -> f32 {
        (value as f32 + 0.5)
            .map(
                <ValidatingIntSliderCallbacks as IntSliderCallbacks<Cx>>::min_inclusive(self) as f32
                    ..<ValidatingIntSliderCallbacks as IntSliderCallbacks<Cx>>::max_inclusive(self)
                        as f32
                        + 1.0,
                0.0..1.0,
            )
            .clamp(0.0, 1.0)
    }

    fn to_value(&self, slider_progress: f32) -> i32 {
        slider_progress
            .map(
                0.0..1.0,
                <ValidatingIntSliderCallbacks as IntSliderCallbacks<Cx>>::min_inclusive(self) as f32
                    ..<ValidatingIntSliderCallbacks as IntSliderCallbacks<Cx>>::max_inclusive(self)
                        as f32
                        + 1.0,
            )
            .floor() as i32
    }

    fn i32_validate(&self, value: Option<i32>) -> Option<i32> {
        value.filter(|&value| value >= self.min && value <= self.max)
    }

    fn with_modifier<R, IR, RI, F, ToP, ToV>(
        &self,
        progress_to_value: IR,
        value_to_progress: RI,
    ) -> impl SliderCallbacks<R, Cx>
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

        impl<R, IR, RI, F, ToP, ToV, Cx> SliderCallbacks<R, Cx> for Impl<IR, RI, F, ToP, ToV>
        where
            Cx: ProvideTextTy,
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

        impl<R, IR, RI, F, ToP, ToV, Cx> Callbacks<R, Cx> for Impl<IR, RI, F, ToP, ToV>
        where
            Cx: ProvideTextTy,
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
            i32_validate: |value| {
                <ValidatingIntSliderCallbacks as IntSliderCallbacks<Cx>>::i32_validate(self, value)
            },
            to_slider_progress: |value| IntSliderCallbacks::<Cx>::to_slider_progress(self, value),
            to_value: |value| IntSliderCallbacks::<Cx>::to_value(self, value),
        }
    }
}

impl<Cx> SliderCallbacks<i32, Cx> for ValidatingIntSliderCallbacks
where
    Cx: ProvideTextTy,
{
    fn to_slider_progress(&self, value: i32) -> f32 {
        <ValidatingIntSliderCallbacks as IntSliderCallbacks<Cx>>::to_slider_progress(self, value)
    }

    fn to_value(&self, slider_progress: f32) -> i32 {
        <ValidatingIntSliderCallbacks as IntSliderCallbacks<Cx>>::to_value(self, slider_progress)
    }
}

impl<Cx> Callbacks<i32, Cx> for ValidatingIntSliderCallbacks
where
    Cx: ProvideTextTy,
{
    fn validate(&self, value: Option<i32>) -> Option<i32> {
        <ValidatingIntSliderCallbacks as IntSliderCallbacks<Cx>>::i32_validate(self, value)
    }
}
