use super::*;

pub struct ValidatingIntSliderCallbacks {
    min: i32,
    max: i32,
}

impl<Txt, TxtStyle> IntSliderCallbacks<Txt, TxtStyle> for ValidatingIntSliderCallbacks {
    fn min_inclusive(&self) -> i32 {
        self.min
    }

    fn max_inclusive(&self) -> i32 {
        self.max
    }

    fn i32_validate(&self, value: Option<i32>) -> Option<i32> {
        match value {
            Some(value) => {
                if value >= self.min && value <= self.max {
                    Some(value)
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

impl<Txt, TxtStyle> SliderCallbacks<i32, Txt, TxtStyle> for ValidatingIntSliderCallbacks {
    fn to_slider_progress(&self, value: i32) -> f32 {
        <ValidatingIntSliderCallbacks as IntSliderCallbacks<Txt, TxtStyle>>::to_slider_progress(self, value)
    }

    fn to_value(&self, slider_progress: f32) -> i32 {
        <ValidatingIntSliderCallbacks as IntSliderCallbacks<Txt, TxtStyle>>::to_value(self, slider_progress)
    }
}

impl<Txt, TxtStyle> Callbacks<i32, Txt, TxtStyle> for ValidatingIntSliderCallbacks {
    fn validate(&self, value: Option<i32>) -> Option<i32> {
        <ValidatingIntSliderCallbacks as IntSliderCallbacks<Txt, TxtStyle>>::i32_validate(self, value)
    }
}
