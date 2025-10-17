use std::{collections::HashMap, hash::Hash};

use rimecraft_text::ProvideTextTy;

use crate::{
    SimpleOption,
    callbacks::{Callbacks, ValueSetter, ty::CyclingCallbacks},
};

pub struct AlternateValuesSupportingCyclingCallbacks<K, T, Cx>
where
    Cx: ProvideTextTy,
{
    pub values: HashMap<K, Vec<T>>,
    pub condition: Box<dyn Fn() -> K>,
    pub value_setter: Box<ValueSetter<T, Cx>>,
    _phantom: std::marker::PhantomData<Cx>,
}

impl<K, V, Cx> AlternateValuesSupportingCyclingCallbacks<K, V, Cx>
where
    Cx: ProvideTextTy,
{
    pub fn new<Condition, F>(
        values: HashMap<K, Vec<V>>,
        condition: Condition,
        value_setter: F,
    ) -> Self
    where
        F: Fn(&mut SimpleOption<V, Cx>, &V) + 'static,
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

impl<V, Cx> AlternateValuesSupportingCyclingCallbacks<bool, V, Cx>
where
    Cx: ProvideTextTy,
{
    pub fn new_binary<Condition, F>(
        true_values: Vec<V>,
        false_values: Vec<V>,
        condition: Condition,
        value_setter: F,
    ) -> Self
    where
        F: Fn(&mut SimpleOption<V, Cx>, &V) + 'static,
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

impl<K, V, Cx> Callbacks<V, Cx> for AlternateValuesSupportingCyclingCallbacks<K, V, Cx>
where
    K: Hash + Eq,
    Cx: ProvideTextTy,
    V: PartialEq + Clone,
{
    fn validate(&self, value: &V) -> Option<V> {
        let key = (self.condition)();
        match self.values.get(&key) {
            Some(values) if values.contains(value) => Some(value.clone()),
            _ => None,
        }
    }
}

impl<K, V, Cx> CyclingCallbacks<V, Cx> for AlternateValuesSupportingCyclingCallbacks<K, V, Cx>
where
    K: Hash + Eq,
    Cx: ProvideTextTy,
    V: PartialEq + Clone,
{
    fn get_values(&self) {
        let key = (self.condition)();
        self.values.get(&key);
        todo!()
    }

    fn value_setter(&self) -> &ValueSetter<V, Cx> {
        self.value_setter.as_ref()
    }
}
