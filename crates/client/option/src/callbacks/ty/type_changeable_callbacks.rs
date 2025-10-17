use rimecraft_text::ProvideTextTy;

use crate::callbacks::ty::{CyclingCallbacks, SliderCallbacks};

pub trait TypeChangeableCallbacks<V, Cx>: CyclingCallbacks<V, Cx> + SliderCallbacks<V, Cx>
where
    Cx: ProvideTextTy,
{
    fn is_cycling(&self) -> bool;
}
