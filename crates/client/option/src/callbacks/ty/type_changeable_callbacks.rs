use rimecraft_text::ProvideTextTy;

use crate::callbacks::ty::{CyclingCallbacks, SliderCallbacks};

pub trait TypeChangeableCallbacks<T, Txt>:
    CyclingCallbacks<T, Txt> + SliderCallbacks<T, Txt>
where
    Txt: ProvideTextTy,
{
    fn is_cycling(&self) -> bool;
}
