use rimecraft_text::ProvideTextTy;

use crate::callbacks::{Callbacks, ty::CyclingCallbacks};

#[derive(Debug, Default, Clone)]
pub struct PotentialValuesBasedCallbacks<V> {
    pub values: Vec<V>,
}

impl<V> PotentialValuesBasedCallbacks<V> {
    pub fn new(values: Vec<V>) -> Self {
        Self { values }
    }
}

impl<V, Cx> CyclingCallbacks<V, Cx> for PotentialValuesBasedCallbacks<V>
where
    Cx: ProvideTextTy,
    V: PartialEq + Clone,
{
    fn get_values(&self) {
        todo!()
    }
}

impl<V, Cx> Callbacks<V, Cx> for PotentialValuesBasedCallbacks<V>
where
    Cx: ProvideTextTy,
    V: PartialEq + Clone,
{
    fn validate(&self, value: &V) -> Option<V> {
        self.values.contains(value).then(|| value.clone())
    }
}
