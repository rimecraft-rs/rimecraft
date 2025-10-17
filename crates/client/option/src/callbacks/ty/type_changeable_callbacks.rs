use rimecraft_text::ProvideTextTy;

use crate::callbacks::ty::{CyclingCallbacks, SliderCallbacks};

pub trait TypeChangeableCallbacks<T, Cx>: CyclingCallbacks<T, Cx> + SliderCallbacks<T, Cx>
where
    Cx: ProvideTextTy,
{
    fn is_cycling(&self) -> bool;
}
