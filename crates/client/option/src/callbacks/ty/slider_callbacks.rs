use rimecraft_text::ProvideTextTy;

use crate::callbacks::Callbacks;

pub trait SliderCallbacks<V, Cx>: Callbacks<V, Cx>
where
    Cx: ProvideTextTy,
{
    fn to_slider_progress(&self, value: V) -> f32;

    fn to_value(&self, slider_progress: f32) -> V;

    fn applies_values_immediately(&self) -> bool {
        true
    }
}
