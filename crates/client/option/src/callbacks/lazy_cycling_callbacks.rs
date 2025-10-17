use rimecraft_text::ProvideTextTy;

use crate::callbacks::{Callbacks, ty::CyclingCallbacks};

type ValuesFn<T> = Box<dyn Fn() -> Vec<T>>;
type ValidateFn<T> = Box<dyn Fn(Option<&T>) -> bool>;

pub struct LazyCyclingCallbacks<T, Cx>
where
    Cx: ProvideTextTy,
{
    pub values: ValuesFn<T>,
    pub validate: ValidateFn<T>,
    _phantom: std::marker::PhantomData<Cx>,
}

impl<T, Cx> LazyCyclingCallbacks<T, Cx>
where
    Cx: ProvideTextTy,
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

impl<T, Cx> Callbacks<T, Cx> for LazyCyclingCallbacks<T, Cx>
where
    Cx: ProvideTextTy,
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

impl<T, Cx> CyclingCallbacks<T, Cx> for LazyCyclingCallbacks<T, Cx>
where
    Cx: ProvideTextTy,
{
    fn get_values(&self) {
        (self.values)();
        todo!()
    }
}
