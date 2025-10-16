mod lazy_cycling_callbacks;
mod potential_values_based_callbacks;
mod suppliable_int_callbacks;
mod validating_int_slider_callbacks;

pub mod ty;

pub use lazy_cycling_callbacks::*;
pub use potential_values_based_callbacks::*;
pub use suppliable_int_callbacks::*;
pub use validating_int_slider_callbacks::*;

use crate::SimpleOption;

use rimecraft_text::ProvideTextTy;

pub type ChangeCallback<T> = dyn Fn(Option<T>);

pub type ValueSetter<T, Txt> = fn(&mut SimpleOption<T, Txt>, value: Option<T>);

pub trait Callbacks<T, Txt>
where
    Txt: ProvideTextTy,
{
    fn validate(&self, value: Option<T>) -> Option<T>;
}
