use rimecraft_text::ProvideTextTy;

use crate::callbacks::{Callbacks, ty::CyclingCallbacks};

type ValuesFn<V> = Box<dyn Fn() -> Vec<V>>;
type ValidateFn<V> = Box<dyn Fn(&V) -> bool>;

pub struct LazyCyclingCallbacks<T, Cx>
where
    Cx: ProvideTextTy,
{
    pub values: ValuesFn<T>,
    pub validate: ValidateFn<T>,
    _phantom: std::marker::PhantomData<Cx>,
}

impl<V, Cx> LazyCyclingCallbacks<V, Cx>
where
    Cx: ProvideTextTy,
{
    pub fn new<Values, Validate>(values: Values, validate: Validate) -> Self
    where
        Values: Fn() -> Vec<V> + 'static,
        Validate: Fn(&V) -> bool + 'static,
    {
        Self {
            values: Box::new(values),
            validate: Box::new(validate),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<V, Cx> Callbacks<V, Cx> for LazyCyclingCallbacks<V, Cx>
where
    Cx: ProvideTextTy,
    V: Clone,
{
    fn validate(&self, value: &V) -> Option<V> {
        (self.validate)(value).then(|| value.clone())
    }
}

impl<V, Cx> CyclingCallbacks<V, Cx> for LazyCyclingCallbacks<V, Cx>
where
    Cx: ProvideTextTy,
    V: Clone,
{
    fn get_values(&self) {
        (self.values)();
        todo!()
    }
}
