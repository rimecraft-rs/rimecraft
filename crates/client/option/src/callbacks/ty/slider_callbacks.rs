use rimecraft_text::ProvideTextTy;

use crate::callbacks::Callbacks;

/// A slider callback for generic value types.
pub trait SliderCallbacks<V, Cx>: Callbacks<V, Cx>
where
    Cx: ProvideTextTy,
{
    /// Indicates whether changes to the slider apply values immediately.
    fn applies_values_immediately(&self) -> bool {
        true
    }

    /// Converts a value to slider progress (0.0 to 1.0).
    fn to_slider_progress(&self, value: V) -> f32;

    /// Converts slider progress (0.0 to 1.0) to a value.
    fn to_value(&self, slider_progress: f32) -> V;
}
