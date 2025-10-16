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

pub type ValueSetter<T, Txt> = dyn Fn(&mut SimpleOption<T, Txt>, Option<T>);

pub trait Callbacks<T, Txt>
where
    Txt: ProvideTextTy,
{
    fn validate(&self, value: Option<T>) -> Option<T>;
}
