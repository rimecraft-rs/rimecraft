use rimecraft_text::ProvideTextTy;

use crate::callbacks::Callbacks;

pub trait SliderCallbacks<T, Txt>: Callbacks<T, Txt>
where
    Txt: ProvideTextTy,
{
    fn to_slider_progress(&self, value: T) -> f32;

    fn to_value(&self, slider_progress: f32) -> T;
}
