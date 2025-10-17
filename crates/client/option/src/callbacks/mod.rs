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

pub type ValueSetter<V, Cx> = dyn Fn(&mut SimpleOption<V, Cx>, &V);

pub trait Callbacks<V, Cx>
where
    Cx: ProvideTextTy,
{
    fn validate(&self, value: &V) -> Option<V>;
}
