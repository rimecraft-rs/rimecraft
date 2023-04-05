pub mod entry;
pub mod registries;
pub mod tag;
pub mod wrapper;

use std::{fmt::Display, slice::Iter, collections::HashMap};

use datafixerupper::serialization::{DynamicOps, Keyable, Lifecycle};

use crate::util::{collection::IndexedIterable, Identifier};

use self::entry::RegistryEntry;

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

    fn get_entry_lifecycle<'a>(&'a self, entry: &'a T) -> &Lifecycle;
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
    fn set(&mut self, id: usize, key: RegistryKey<T>, object: T, lifecycle: Lifecycle) -> &RegistryEntry<T>;
    fn add(&mut self, key: RegistryKey<T>, object: T, lifecycle: Lifecycle) -> &RegistryEntry<T>;
    fn is_empty(&self) -> bool;
}

pub struct SimpleRegistry<'r, T> {
    key: &'r RegistryKey<SimpleRegistry<'r, T>>,
    entries: Vec<(RegistryEntry<'r, T>, RegistryKey<T>, Identifier, Lifecycle)>,
    lifecycle: Lifecycle,
    
}

impl<T: PartialEq> IndexedIterable<T> for SimpleRegistry<'_, T> {
    fn get_raw_id<'a>(&'a self, object: &'a T) -> Option<usize> {
        todo!()
    }

    fn get_from_raw_id(&self, id: usize) -> Option<&T> {
        todo!()
    }

    fn get_from_raw_id_mut(&mut self, id: usize) -> Option<&mut T> {
        todo!()
    }

    fn size(&self) -> usize {
        todo!()
    }

    fn iter(&self) -> std::option::Iter<T> {
        todo!()
    }

    fn iter_mut(&mut self) -> std::option::IterMut<T> {
        todo!()
    }
}

impl<T: PartialEq> Registry<T> for SimpleRegistry<'_, T> {
    fn get_self_key(&self) -> &RegistryKey<Self> {
        todo!()
    }

    fn get_id<'a>(&'a self, obj: &'a T) -> Option<&'a Identifier> {
        todo!()
    }

    fn get_key<'a>(&'a self, obj: &'a T) -> Option<&'a RegistryKey<T>> {
        todo!()
    }

    fn get_from_key<'a>(&'a self, key: &RegistryKey<T>) -> Option<&'a T> {
        todo!()
    }

    fn get_from_id<'a>(&'a self, id: &Identifier) -> Option<&'a T> {
        todo!()
    }

    fn get_entry_lifecycle<'a>(&'a self, entry: &'a T) -> &Lifecycle {
        todo!()
    }

    fn get_lifecycle(&self) -> &Lifecycle {
        todo!()
    }

    fn get_ids(&self) -> Vec<&Identifier> {
        todo!()
    }

    fn get_entry_set(&self) -> Vec<(&RegistryKey<T>, &T)> {
        todo!()
    }

    fn get_keys(&self) -> Vec<&RegistryKey<T>> {
        todo!()
    }

    fn contains_id(&self, id: &Identifier) -> bool {
        todo!()
    }

    fn contains(&self, key: &RegistryKey<T>) -> bool {
        todo!()
    }

    fn freeze(&mut self) {
        todo!()
    }
}