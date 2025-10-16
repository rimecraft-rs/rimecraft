use rimecraft_text::ProvideTextTy;

use crate::callbacks::{Callbacks, ty::CyclingCallbacks};

#[derive(Debug, Default, Clone)]
pub struct PotentialValuesBasedCallbacks<T> {
    pub values: Vec<T>,
}

impl<T> PotentialValuesBasedCallbacks<T> {
    pub fn new(values: Vec<T>) -> Self {
        Self { values }
    }
}

impl<T, Txt> CyclingCallbacks<T, Txt> for PotentialValuesBasedCallbacks<T>
where
    Txt: ProvideTextTy,
    T: PartialEq,
{
    fn get_values(&self) {
        todo!()
    }
}

impl<T, Txt> Callbacks<T, Txt> for PotentialValuesBasedCallbacks<T>
where
    Txt: ProvideTextTy,
    T: PartialEq,
{
    fn validate(&self, value: Option<T>) -> Option<T> {
        match value {
            Some(ref v) if self.values.contains(v) => value,
            _ => None,
        }
    }
}
