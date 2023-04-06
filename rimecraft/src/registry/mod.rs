pub mod entry;
pub mod registries;
pub mod tag;
pub mod wrapper;

use std::{fmt::Display, collections::HashMap};
use datafixerupper::serialization::Lifecycle;
use crate::util::{collection::IndexedIterable, Identifier, self};
use self::{entry::RegistryEntry, tag::TagKey};

pub struct RegistryKey<T> {
    registry: Identifier,
    value: Identifier,
    _none: Option<T>,
}

impl<T> RegistryKey<T> {
    pub fn new(registry: Identifier, value: Identifier) -> Self {
        Self {
            registry,
            value,
            _none: None,
        }
    }

    pub fn of<V>(registry: &RegistryKey<V>, value: Identifier) -> Self
    where
        V: Registry<T>,
    {
        Self::new(registry.value.clone(), value)
    }

    pub fn of_registry<V>(registry: Identifier) -> Self
    where
        T: Registry<V>,
    {
        Self::new(registries::root_key(), registry)
    }

    pub fn is_of<V, A>(&self, registry: &RegistryKey<V>) -> bool
    where
        V: Registry<A>,
    {
        self.registry.eq(registry.get_value())
    }

    pub fn try_cast<V, A>(&self, registry_ref: &RegistryKey<V>) -> Option<RegistryKey<V>>
    where
        V: Registry<A>,
    {
        if self.is_of(registry_ref) {
            Some(RegistryKey {
                registry: self.registry.clone(),
                value: self.value.clone(),
                _none: None,
            })
        } else {
            None
        }
    }

    pub fn get_value(&self) -> &Identifier {
        &self.value
    }

    pub fn get_registry(&self) -> &Identifier {
        &self.registry
    }
}

impl<T> Display for RegistryKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RegistryKey[")?;
        self.registry.fmt(f)?;
        f.write_str(" / ")?;
        self.value.fmt(f)?;
        f.write_str("]")?;
        std::fmt::Result::Ok(())
    }
}

impl<T> Clone for RegistryKey<T> {
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            value: self.value.clone(),
            _none: None,
        }
    }
}

impl<T> PartialEq for RegistryKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.registry == other.registry && self.value == other.value
    }
}

pub trait Registry<T>: 
// Keyable + 
IndexedIterable<T> {
    // fn keys<V>(&self, ops: &impl DynamicOps<V>) -> Iter<V> {
    //     todo!()
    // }

    fn get_self_key(&self) -> &RegistryKey<Self>;

    fn get_id<'a>(&'a self, obj: &'a T) -> Option<&'a Identifier>;
    fn get_key<'a>(&'a self, obj: &'a T) -> Option<&'a RegistryKey<T>>;

    fn get_from_key<'a>(&'a self, key: &RegistryKey<T>) -> Option<&'a T>;
    fn get_from_id<'a>(&'a self, id: &Identifier) -> Option<&'a T>;

    fn get_entry_lifecycle<'a>(&'a self, entry: &'a T) -> Option<&Lifecycle>;
    fn get_lifecycle(&self) -> &Lifecycle;

    fn get_ids(&self) -> Vec<&Identifier>;
    fn get_entry_set(&self) -> Vec<(&RegistryKey<T>, &T)>;
    fn get_keys(&self) -> Vec<&RegistryKey<T>>;

    fn contains_id(&self, id: &Identifier) -> bool;
    fn contains(&self, key: &RegistryKey<T>) -> bool;

    fn freeze(&mut self);
}

pub trait DefaultedRegistry<T>: Registry<T> {
    fn get_id_default<'a>(&'a self, object: &'a T) -> &'a Identifier;
    fn get_from_id_default(&self, id: &Identifier) -> &T;
    fn get_from_raw_id_default(&self, id: usize) -> &T;
    fn get_default_id(&self) -> Identifier;
}

pub trait MutableRegistry<T>: Registry<T> {
    fn set(&mut self, id: usize, key: RegistryKey<T>, object: T, lifecycle: Lifecycle) -> &RegistryEntry<T, Self>;
    fn add(&mut self, key: RegistryKey<T>, object: T, lifecycle: Lifecycle) -> &RegistryEntry<T, Self>;
    fn is_empty(&self) -> bool;
}

pub struct SimpleRegistry<'r, T: PartialEq> {
    key: &'r RegistryKey<Self>,
    entries: Vec<(RegistryEntry<T, Self>, RegistryKey<T>, Identifier, Lifecycle)>,
    lifecycle: Lifecycle,
    tags: HashMap<TagKey<T, Self>, Vec<usize>>,
    frozen: bool,
}

impl<'r, T: PartialEq> SimpleRegistry<'r, T> {
    pub fn new(key: &'r RegistryKey<Self>, lifecycle: Lifecycle) -> Self {
        Self { key, entries: Vec::new(), lifecycle, tags: HashMap::new(), frozen: false }
    }
}

impl<T: PartialEq> IndexedIterable<T> for SimpleRegistry<'_, T> {
    fn get_raw_id<'a>(&'a self, object: &'a T) -> Option<usize> {
        for e in self.entries.iter().enumerate() {
            if e.1.0.value().is_some() && e.1.0.value().unwrap() == object {
                return Some(e.0);
            }
        }
        None
    }

    fn get_from_raw_id(&self, id: usize) -> Option<&T> {
        match self.entries.get(id).map(|t| t.0.value()) {
            Some(Some(a)) => Some(a),
            _ => None,
        }
    }

    fn get_from_raw_id_mut(&mut self, id: usize) -> Option<&mut T> {
        match self.entries.get_mut(id).map(|t| t.0.value_mut()) {
            Some(Some(a)) => Some(a),
            _ => None,
        }
    }

    fn size(&self) -> usize {
        self.entries.len()
    }

    fn vec(&self) -> Vec<&T> {
        self.entries.iter().map(|e| e.0.value()).filter(|o| o.is_some()).map(|o| o.unwrap()).collect()
    }

    fn vec_mut(&mut self) -> Vec<&mut T> {
        self.entries.iter_mut().map(|e| e.0.value_mut()).filter(|o| o.is_some()).map(|o| o.unwrap()).collect()
    }
}

impl<T: PartialEq> Registry<T> for SimpleRegistry<'_, T> {
    fn get_self_key(&self) -> &RegistryKey<Self> {
        self.key
    }

    fn get_id<'a>(&'a self, obj: &'a T) -> Option<&'a Identifier> {
        for entry in &self.entries {
            if let Some(v) = entry.0.value() {
                if obj == v {
                    return Some(&entry.2);
                }
            }
        }
        None
    }

    fn get_key<'a>(&'a self, obj: &'a T) -> Option<&'a RegistryKey<T>> {
        for entry in &self.entries {
            if let Some(v) = entry.0.value() {
                if obj == v {
                    return Some(&entry.1);
                }
            }
        }
        None
    }

    fn get_from_key<'a>(&'a self, key: &RegistryKey<T>) -> Option<&'a T> {
        for entry in &self.entries {
            if &entry.1 == key {
                return entry.0.value()
            }
        }
        None
    }

    fn get_from_id<'a>(&'a self, id: &Identifier) -> Option<&'a T> {
        for entry in &self.entries {
            if &entry.2 == id {
                return entry.0.value()
            }
        }
        None
    }

    fn get_entry_lifecycle<'a>(&'a self, entry: &'a T) -> Option<&Lifecycle> {
        for e in &self.entries {
            if let Some(v) = e.0.value() {
                if entry == v {
                    return Some(&e.3);
                }
            }
        }
        None
    }

    fn get_lifecycle(&self) -> &Lifecycle {
        &self.lifecycle
    }

    fn get_ids(&self) -> Vec<&Identifier> {
        self.entries.iter().map(|t| &t.2).collect()
    }

    fn get_entry_set(&self) -> Vec<(&RegistryKey<T>, &T)> {
        self.entries.iter().filter(|t| t.0.value().is_some()).map(|t| (&t.1, t.0.value().unwrap())).collect()
    }

    fn get_keys(&self) -> Vec<&RegistryKey<T>> {
        self.entries.iter().map(|t| &t.1).collect()
    }

    fn contains_id(&self, id: &Identifier) -> bool {
        self.entries.iter().any(|p| &p.2 == id)
    }

    fn contains(&self, key: &RegistryKey<T>) -> bool {
        self.entries.iter().any(|p| &p.1 == key)
    }

    fn freeze(&mut self) {
        if self.frozen {
            return;
        }
        self.frozen = true;
    }
}