use super::*;

pub struct SuppliableIntCallbacks {
    min_boundary: fn() -> i32,
    max_boundary: fn() -> i32,
}

impl<Txt, TxtStyle> TypeChangeableCallbacks<i32, Txt, TxtStyle> for SuppliableIntCallbacks {
    fn is_cycling(&self) -> bool {
        true
    }
}

impl<Txt, TxtStyle> CyclingCallbacks<i32, Txt, TxtStyle> for SuppliableIntCallbacks {
    fn get_values(&self) -> () {
        todo!()
    }
}

impl<Txt, TxtStyle> IntSliderCallbacks<Txt, TxtStyle> for SuppliableIntCallbacks {
    fn min_inclusive(&self) -> i32 {
        (self.min_boundary)()
    }

    fn max_inclusive(&self) -> i32 {
        (self.max_boundary)()
    }

    fn i32_validate(&self, value: Option<i32>) -> Option<i32> {
        match value {
            Some(value) => Some(rimecraft_math::clamp(
                value as f32,
                <SuppliableIntCallbacks as IntSliderCallbacks<Txt, TxtStyle>>::min_inclusive(self) as f32,
                <SuppliableIntCallbacks as IntSliderCallbacks<Txt, TxtStyle>>::max_inclusive(self) as f32,
            ) as i32),
            None => None,
        }
    }
}

impl<Txt, TxtStyle> SliderCallbacks<i32, Txt, TxtStyle> for SuppliableIntCallbacks {
    fn to_slider_progress(&self, value: i32) -> f32 {
        <SuppliableIntCallbacks as IntSliderCallbacks<Txt, TxtStyle>>::to_slider_progress(self, value)
    }

    fn to_value(&self, slider_progress: f32) -> i32 {
        <SuppliableIntCallbacks as IntSliderCallbacks<Txt, TxtStyle>>::to_value(self, slider_progress)
    }
}

impl<Txt, TxtStyle> Callbacks<i32, Txt, TxtStyle> for SuppliableIntCallbacks {
    fn validate(&self, value: Option<i32>) -> Option<i32> {
        <SuppliableIntCallbacks as IntSliderCallbacks<Txt, TxtStyle>>::i32_validate(self, value)
    }
}