use rimecraft_text::ProvideTextTy;

use crate::callbacks::{Callbacks, ty::CyclingCallbacks};

type ValuesFn<T> = Box<dyn Fn() -> Vec<T>>;
type ValidateFn<T> = Box<dyn Fn(Option<&T>) -> bool>;

pub struct LazyCyclingCallbacks<T, Txt>
where
    Txt: ProvideTextTy,
{
    pub values: ValuesFn<T>,
    pub validate: ValidateFn<T>,
    _phantom: std::marker::PhantomData<Txt>,
}

impl<T, Txt> LazyCyclingCallbacks<T, Txt>
where
    Txt: ProvideTextTy,
{
    pub fn new<Values, Validate>(values: Values, validate: Validate) -> Self
    where
        Values: Fn() -> Vec<T> + 'static,
        Validate: Fn(Option<&T>) -> bool + 'static,
    {
        Self {
            values: Box::new(values),
            validate: Box::new(validate),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, Txt> Callbacks<T, Txt> for LazyCyclingCallbacks<T, Txt>
where
    Txt: ProvideTextTy,
{
    fn validate(&self, value: Option<T>) -> Option<T> {
        match value {
            Some(ref v) if (self.validate)(Some(v)) => value,
            Some(_) => None,
            None if (self.validate)(None) => None,
            None => None,
        }
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
