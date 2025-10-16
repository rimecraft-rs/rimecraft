use rimecraft_text::ProvideTextTy;

use crate::callbacks::{Callbacks, ValueSetter};

pub trait CyclingCallbacks<T, Txt>: Callbacks<T, Txt>
where
    Txt: ProvideTextTy,
{
    fn get_values(&self); // CyclingButtonWidget.Values<T>

    fn value_setter(&self) -> &ValueSetter<T, Txt> {
        &|option, value| option.set_value(value)
    }
}
