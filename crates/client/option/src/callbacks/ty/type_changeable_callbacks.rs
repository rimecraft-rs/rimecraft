use rimecraft_text::ProvideTextTy;

use crate::callbacks::ty::{CyclingCallbacks, SliderCallbacks};

/// An enum representing the mode of an option that can switch between cycling and slider types.
///
/// See: [`TypeChangeableCallbacks`], [`CyclingCallbacks`], [`SliderCallbacks`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TypeChangeableMode {
    /// Cycling mode.
    ///
    /// See: [`CyclingCallbacks`]
    Cycling,
    /// Slider mode.
    ///
    /// See: [`SliderCallbacks`]
    Slider,
}

/// A callback for options that can change between various types:
///
/// - Cycling options (using [`CyclingCallbacks`])
/// - Slider options (using [`SliderCallbacks`])
///
/// See: [`TypeChangeableMode`]
pub trait TypeChangeableCallbacks<V, Cx>: CyclingCallbacks<V, Cx> + SliderCallbacks<V, Cx>
where
    Cx: ProvideTextTy,
{
    /// Returns the current [`TypeChangeableMode`] of the type-changeable option.
    fn type_changeable_mode(&self) -> TypeChangeableMode;
}
