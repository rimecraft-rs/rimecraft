//! Callback implementations and traits.

mod alternate_values_supporting_cycling_callbacks;
mod lazy_cycling_callbacks;
mod potential_values_based_callbacks;
mod suppliable_int_callbacks;
mod validating_int_slider_callbacks;

pub mod ty;

pub use alternate_values_supporting_cycling_callbacks::*;
pub use lazy_cycling_callbacks::*;
pub use potential_values_based_callbacks::*;
pub use suppliable_int_callbacks::*;
pub use validating_int_slider_callbacks::*;

use crate::SimpleOption;

use rimecraft_text::ProvideTextTy;

/// A function that sets the value of a [`SimpleOption`].
pub type ValueSetter<V, Cx> = dyn Fn(&mut SimpleOption<V, Cx>, V);

/// A trait for option callbacks.
pub trait Callbacks<V, Cx>
where
    Cx: ProvideTextTy,
{
    /// Validates the given value, returning `Some(validated_value)` if valid,
    /// or `None` if invalid.
    ///
    /// This function takes ownership of `value` to allow for more flexible validation logic.
    fn validate(&self, value: V) -> Option<V>;
}

/// Creates a boolean [`Callbacks`] instance that supports `true` and `false` values.
pub fn bool<Cx>() -> impl Callbacks<bool, Cx>
where
    Cx: ProvideTextTy,
{
    PotentialValuesBasedCallbacks {
        values: vec![true, false],
    }
}
