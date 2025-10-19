use std::{collections::HashMap, fmt::Debug, hash::Hash};

use rimecraft_text::ProvideTextTy;

use crate::{
    SimpleOption,
    callbacks::{Callbacks, ValueSetter, ty::CyclingCallbacks},
};

/// A callback for cycling options that support alternate value sets based on a condition.
///
/// Can also be named `AlternateValuesSupportingCyclingCallbacks`.
pub struct AltCyclingCallbacks<K, V, Cx>
where
    Cx: ProvideTextTy,
{
    /// A mapping from condition keys to their corresponding value sets.
    pub values: HashMap<K, Vec<V>>,
    /// A function that determines the current condition key.
    pub condition: Box<dyn Fn() -> K>,
    /// A [`ValueSetter`] that sets the value of the option.
    pub value_setter: Box<ValueSetter<V, Cx>>,
}

impl<K, V, Cx> Debug for AltCyclingCallbacks<K, V, Cx>
where
    K: Debug + Hash + Eq,
    V: Debug,
    Cx: ProvideTextTy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlternateValuesSupportingCyclingCallbacks")
            .field("values", &self.values)
            .finish()
    }
}

impl<K, V, Cx> AltCyclingCallbacks<K, V, Cx>
where
    Cx: ProvideTextTy,
{
    /// Creates a new [`AltCyclingCallbacks`].
    pub fn new<Condition, F>(
        values: HashMap<K, Vec<V>>,
        condition: Condition,
        value_setter: F,
    ) -> Self
    where
        F: Fn(&mut SimpleOption<V, Cx>, V) + 'static,
        Condition: Fn() -> K + 'static,
    {
        Self {
            values,
            condition: Box::new(condition),
            value_setter: Box::new(value_setter),
        }
    }
}

impl<V, Cx> AltCyclingCallbacks<bool, V, Cx>
where
    Cx: ProvideTextTy,
{
    /// Creates a new [`AltCyclingCallbacks`] that switches between two sets of values
    /// based on a boolean condition.
    pub fn new_binary<Condition, F>(
        true_values: Vec<V>,
        false_values: Vec<V>,
        condition: Condition,
        value_setter: F,
    ) -> Self
    where
        F: Fn(&mut SimpleOption<V, Cx>, V) + 'static,
        Condition: Fn() -> bool + 'static,
    {
        let mut values = HashMap::new();

        values.insert(true, true_values);
        values.insert(false, false_values);

        Self {
            values,
            condition: Box::new(condition),
            value_setter: Box::new(value_setter),
        }
    }
}

impl<K, V, Cx> Callbacks<V, Cx> for AltCyclingCallbacks<K, V, Cx>
where
    K: Hash + Eq,
    Cx: ProvideTextTy,
    V: PartialEq,
{
    fn validate(&self, value: V) -> Option<V> {
        let key = (self.condition)();
        match self.values.get(&key) {
            Some(values) if values.contains(&value) => Some(value),
            _ => None,
        }
    }
}

impl<K, V, Cx> CyclingCallbacks<V, Cx> for AltCyclingCallbacks<K, V, Cx>
where
    K: Hash + Eq,
    Cx: ProvideTextTy,
    V: PartialEq,
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
