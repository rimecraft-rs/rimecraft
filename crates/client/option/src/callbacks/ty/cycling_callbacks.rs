use rimecraft_text::ProvideTextTy;

use crate::callbacks::{Callbacks, ValueSetter};

pub trait CyclingCallbacks<V, Cx>: Callbacks<V, Cx>
where
    Cx: ProvideTextTy,
{
    fn get_values(&self); // CyclingButtonWidget.Values<T>

    fn value_setter(&self) -> &ValueSetter<V, Cx>
    where
        V: Clone + PartialEq,
    {
        &|option, value| option.set_value(value)
    }
}
