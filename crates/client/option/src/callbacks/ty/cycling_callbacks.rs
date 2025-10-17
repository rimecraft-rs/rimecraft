use rimecraft_text::ProvideTextTy;

use crate::callbacks::{Callbacks, ValueSetter};

pub trait CyclingCallbacks<T, Cx>: Callbacks<T, Cx>
where
    Cx: ProvideTextTy,
{
    fn get_values(&self); // CyclingButtonWidget.Values<T>

    fn value_setter(&self) -> &ValueSetter<T, Cx> {
        &|option, value| option.set_value(value)
    }
}
