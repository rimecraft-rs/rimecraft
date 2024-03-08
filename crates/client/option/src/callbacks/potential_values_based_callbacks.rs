use super::*;

pub struct PotentialValuesBasedCallbacks<T> {
    values: Vec<T>,
}

impl<T, Txt, TxtStyle> CyclingCallbacks<T, Txt, TxtStyle> for PotentialValuesBasedCallbacks<T> {
    fn get_values(&self) -> () {
        todo!()
    }
}

impl<T, Txt, TxtStyle> Callbacks<T, Txt, TxtStyle> for PotentialValuesBasedCallbacks<T> {
    fn validate(&self, value: Option<T>) -> Option<T> {
        todo!()
    }
}