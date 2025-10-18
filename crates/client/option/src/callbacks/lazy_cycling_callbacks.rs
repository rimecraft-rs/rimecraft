use std::fmt::Debug;

use rimecraft_text::ProvideTextTy;

use crate::callbacks::{Callbacks, ty::CyclingCallbacks};

type ValuesFn<V> = Box<dyn Fn() -> Vec<V>>;
type ValidateFn<V> = Box<dyn Fn(V) -> Option<V>>;

/// A lazy cycling callback that provides values and validation for options with values of type `V`.
pub struct LazyCyclingCallbacks<V, Cx>
where
    Cx: ProvideTextTy,
{
    /// A function that returns the list of possible values.
    pub values: ValuesFn<V>,
    /// A function that validates a given value.
    pub validate: ValidateFn<V>,
    _phantom: std::marker::PhantomData<Cx>,
}

impl<V, Cx> Debug for LazyCyclingCallbacks<V, Cx>
where
    Cx: ProvideTextTy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LazyCyclingCallbacks").finish()
    }
}

impl<V, Cx> LazyCyclingCallbacks<V, Cx>
where
    Cx: ProvideTextTy,
{
    /// Creates a new [`LazyCyclingCallbacks`] with the given values and validation functions.
    pub fn new<Values, Validate>(values: Values, validate: Validate) -> Self
    where
        Values: Fn() -> Vec<V> + 'static,
        Validate: Fn(V) -> Option<V> + 'static,
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
    fn validate(&self, value: V) -> Option<V> {
        (self.validate)(value)
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
