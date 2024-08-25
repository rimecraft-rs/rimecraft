use super::*;

struct ValidatingIntSliderCallbacks {
    min: i32,
    max: i32,
}

impl<Txt> IntSliderCallbacks<Txt> for ValidatingIntSliderCallbacks
where
    Txt: ProvideTextTy,
{
    fn min_inclusive(&self) -> i32 {
        self.min
    }

    fn max_inclusive(&self) -> i32 {
        self.max
    }

    fn i32_validate(&self, value: Option<i32>) -> Option<i32> {
        value.filter(|&value| value >= self.min && value <= self.max)
    }
}

impl<Txt> SliderCallbacks<i32, Txt> for ValidatingIntSliderCallbacks
where
    Txt: ProvideTextTy,
{
    fn to_slider_progress(&self, value: i32) -> f32 {
        <ValidatingIntSliderCallbacks as IntSliderCallbacks<Txt>>::to_slider_progress(self, value)
    }

    fn to_value(&self, slider_progress: f32) -> i32 {
        <ValidatingIntSliderCallbacks as IntSliderCallbacks<Txt>>::to_value(self, slider_progress)
    }
}

impl<Txt> Callbacks<i32, Txt> for ValidatingIntSliderCallbacks
where
    Txt: ProvideTextTy,
{
    fn validate(&self, value: Option<i32>) -> Option<i32> {
        <ValidatingIntSliderCallbacks as IntSliderCallbacks<Txt>>::i32_validate(self, value)
    }
}
