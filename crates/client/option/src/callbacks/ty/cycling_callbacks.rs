use rimecraft_text::ProvideTextTy;

use crate::callbacks::{Callbacks, ValueSetter};

/// A callback that provides cycling behavior for options with values of type `V`.
pub trait CyclingCallbacks<V, Cx>: Callbacks<V, Cx>
where
    Cx: ProvideTextTy,
{
    fn values(&self); // CyclingButtonWidget.Values<T>

    /// Returns a reference to the [`ValueSetter`].
    fn value_setter(&self) -> &ValueSetter<V, Cx>
    where
        V: Clone + PartialEq,
    {
        &|option, value| option.set_value(value)
    }
}
