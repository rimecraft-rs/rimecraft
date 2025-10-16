use std::{collections::HashMap, hash::Hash};

use rimecraft_text::ProvideTextTy;

use crate::{
    SimpleOption,
    callbacks::{Callbacks, ValueSetter, ty::CyclingCallbacks},
};

pub struct AlternateValuesSupportingCyclingCallbacks<K, T, Txt>
where
    Txt: ProvideTextTy,
{
    pub values: HashMap<K, Vec<T>>,
    pub condition: Box<dyn Fn() -> K>,
    pub value_setter: Box<ValueSetter<T, Txt>>,
    _phantom: std::marker::PhantomData<Txt>,
}

impl<K, T, Txt> AlternateValuesSupportingCyclingCallbacks<K, T, Txt>
where
    Txt: ProvideTextTy,
{
    pub fn new<Condition, F>(
        values: HashMap<K, Vec<T>>,
        condition: Condition,
        value_setter: F,
    ) -> Self
    where
        F: Fn(&mut SimpleOption<T, Txt>, Option<T>) + 'static,
        Condition: Fn() -> K + 'static,
    {
        Self {
            values,
            condition: Box::new(condition),
            value_setter: Box::new(value_setter),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, Txt> AlternateValuesSupportingCyclingCallbacks<bool, T, Txt>
where
    Txt: ProvideTextTy,
{
    pub fn new_binary<Condition, F>(
        true_values: Vec<T>,
        false_values: Vec<T>,
        condition: Condition,
        value_setter: F,
    ) -> Self
    where
        F: Fn(&mut SimpleOption<T, Txt>, Option<T>) + 'static,
        Condition: Fn() -> bool + 'static,
    {
        let mut values = HashMap::new();

        values.insert(true, true_values);
        values.insert(false, false_values);

        Self {
            values,
            condition: Box::new(condition),
            value_setter: Box::new(value_setter),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<K, T, Txt> Callbacks<T, Txt> for AlternateValuesSupportingCyclingCallbacks<K, T, Txt>
where
    K: Hash + Eq,
    Txt: ProvideTextTy,
    T: PartialEq,
{
    fn validate(&self, value: Option<T>) -> Option<T> {
        let key = (self.condition)();
        match self.values.get(&key) {
            Some(values) => match value {
                Some(ref v) if values.contains(v) => value,
                _ => None,
            },
            None => None,
        }
    }
}

impl<K, T, Txt> CyclingCallbacks<T, Txt> for AlternateValuesSupportingCyclingCallbacks<K, T, Txt>
where
    K: Hash + Eq,
    Txt: ProvideTextTy,
    T: PartialEq,
{
    fn get_values(&self) {
        let key = (self.condition)();
        self.values.get(&key);
        todo!()
    }

    fn value_setter(&self) -> &ValueSetter<T, Txt> {
        self.value_setter.as_ref()
    }
}
