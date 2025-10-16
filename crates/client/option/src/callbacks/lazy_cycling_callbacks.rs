use rimecraft_text::ProvideTextTy;

use crate::callbacks::{Callbacks, ty::CyclingCallbacks};

pub struct LazyCyclingCallbacks<T, Txt>
where
    Txt: ProvideTextTy,
{
    pub values: fn() -> Vec<T>,
    pub validator: fn(Option<T>) -> Option<T>,
    _phantom: std::marker::PhantomData<Txt>,
}

impl<T, Txt> Callbacks<T, Txt> for LazyCyclingCallbacks<T, Txt>
where
    Txt: ProvideTextTy,
{
    fn validate(&self, value: Option<T>) -> Option<T> {
        (self.validator)(value)
    }
}

impl<T, Txt> CyclingCallbacks<T, Txt> for LazyCyclingCallbacks<T, Txt>
where
    Txt: ProvideTextTy,
{
    fn get_values(&self) {
        (self.values)();
        todo!()
    }
}
