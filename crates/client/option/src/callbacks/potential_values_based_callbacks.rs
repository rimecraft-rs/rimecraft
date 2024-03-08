use super::*;

pub struct PotentialValuesBasedCallbacks<T> {
    values: Vec<T>,
}

impl<T, Txt> CyclingCallbacks<T, Txt> for PotentialValuesBasedCallbacks<T>
where
    Txt: ProvideTextTy,
{
    fn get_values(&self) -> () {
        todo!()
    }
}

impl<T, Txt> Callbacks<T, Txt> for PotentialValuesBasedCallbacks<T>
where
    Txt: ProvideTextTy,
{
    fn validate(&self, value: Option<T>) -> Option<T> {
        todo!()
    }
}
