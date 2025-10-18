use rimecraft_text::ProvideTextTy;

use crate::callbacks::{Callbacks, ty::CyclingCallbacks};

/// A callback that provides cycling behavior based on a set of potential values.
#[derive(Debug, Default, Clone)]
pub struct PotentialValuesBasedCallbacks<V> {
    /// The potential values for the option.
    pub values: Vec<V>,
}

impl<V> PotentialValuesBasedCallbacks<V> {
    /// Creates a new [`PotentialValuesBasedCallbacks`] with the given potential values.
    pub fn new(values: Vec<V>) -> Self {
        Self { values }
    }
}

impl<V, Cx> CyclingCallbacks<V, Cx> for PotentialValuesBasedCallbacks<V>
where
    Cx: ProvideTextTy,
    V: PartialEq,
{
    fn get_values(&self) {
        todo!()
    }
}

impl<V, Cx> Callbacks<V, Cx> for PotentialValuesBasedCallbacks<V>
where
    Cx: ProvideTextTy,
    V: PartialEq,
{
    fn validate(&self, value: V) -> Option<V> {
        self.values.contains(&value).then_some(value)
    }
}
