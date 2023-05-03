pub mod entry;
pub mod registries;
pub mod registry_keys;
pub mod tag;
pub mod wrapper;

use self::entry::RegistryEntry;
use crate::{
    registry::entry::ReferenceEntry,
    util::{collection::IndexedIterable, Identifier},
};
use datafixerupper::serialization::Lifecycle;
use log::error;
use std::{fmt::Display, marker::PhantomData};

pub struct RegistryKey<T> {
    pub registry: Identifier,
    pub value: Identifier,
    _phantom: PhantomData<T>,
}

impl<T> RegistryKey<T> {
    pub fn new(registry: Identifier, value: Identifier) -> Self {
        Self {
            registry,
            value,
            _phantom: PhantomData,
        }
    }

    pub fn of(registry: &RegistryKey<Registry<T>>, value: Identifier) -> Self {
        Self::new(registry.value.clone(), value)
    }

    pub fn is_of<A>(&self, registry: &RegistryKey<Registry<A>>) -> bool {
        self.registry.eq(registry.get_value())
    }

    pub fn try_cast<A>(
        &self,
        registry_ref: &RegistryKey<Registry<A>>,
    ) -> Option<RegistryKey<Registry<A>>> {
        if self.is_of(registry_ref) {
            Some(RegistryKey {
                registry: self.registry.clone(),
                value: self.value.clone(),
                _phantom: PhantomData,
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

impl<V> RegistryKey<Registry<V>> {
    pub fn of_registry(registry: Identifier) -> Self {
        Self::new(registries::root_key(), registry)
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
            _phantom: PhantomData,
        }
    }
}

impl<T> PartialEq for RegistryKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.registry == other.registry && self.value == other.value
    }
}

pub struct Registry<T> {
    key: (Identifier, Identifier),
    entries: Vec<(RegistryEntry<T>, Lifecycle)>,
    lifecycle: Lifecycle,
    frozen: bool,
    default_id: Option<Identifier>,
}

impl<T> Registry<T> {
    pub fn new(key: RegistryKey<Self>, lifecycle: Lifecycle, default: Option<Identifier>) -> Self {
        Self {
            key: (key.get_registry().clone(), key.get_value().clone()),
            entries: Vec::new(),
            lifecycle,
            frozen: false,
            default_id: default,
        }
    }

    pub fn get_self_key(&self) -> RegistryKey<Self> {
        RegistryKey::new(self.key.0.clone(), self.key.1.clone())
    }

    pub fn get_entry_from_raw_id(&self, id: usize) -> Option<&RegistryEntry<T>> {
        self.entries.get(id).map(|f| &f.0)
    }

    pub fn get_raw_id_from_key(&self, key: &RegistryKey<T>) -> Option<usize> {
        for entry in self.entries.iter().enumerate() {
            if entry.1 .0.get_key().is_some() && entry.1 .0.get_key().unwrap().eq(key) {
                return Some(entry.0);
            }
        }
        None
    }

    pub fn get_raw_id_from_id(&self, id: &Identifier) -> Option<usize> {
        for entry in self.entries.iter().enumerate() {
            if entry.1 .0.get_key().is_some() && entry.1 .0.get_key().unwrap().get_value().eq(id) {
                return Some(entry.0);
            }
        }
        None
    }

    pub fn get_from_key<'a>(&'a self, key: &RegistryKey<T>) -> Option<&'a T> {
        for entry in &self.entries {
            if entry.0.get_key().is_some() && entry.0.get_key().unwrap() == key {
                return entry.0.value();
            }
        }
        None
    }

    pub fn get_from_id<'a>(&'a self, id: &Identifier) -> Option<&'a T> {
        for entry in &self.entries {
            if entry.0.get_key().is_some() && entry.0.get_key().unwrap().get_value() == id {
                return entry.0.value();
            }
        }
        None
    }

    pub fn get_entry_lifecycle(&self, entry: usize) -> Option<&Lifecycle> {
        self.entries.get(entry).map(|f| &f.1)
    }

    pub fn get_lifecycle(&self) -> &Lifecycle {
        &self.lifecycle
    }

    pub fn get_ids(&self) -> Vec<&Identifier> {
        self.entries
            .iter()
            .filter(|t| t.0.get_key().is_some())
            .map(|t| t.0.get_key().unwrap().get_value())
            .collect()
    }

    pub fn get_entry_set(&self) -> Vec<(&RegistryKey<T>, &T)> {
        self.entries
            .iter()
            .filter(|t| t.0.value().is_some() && t.0.get_key().is_some())
            .map(|t| (t.0.get_key().unwrap(), t.0.value().unwrap()))
            .collect()
    }

    pub fn get_keys(&self) -> Vec<&RegistryKey<T>> {
        self.entries.iter().filter_map(|t| t.0.get_key()).collect()
    }

    pub fn contains_id(&self, id: &Identifier) -> bool {
        self.entries
            .iter()
            .any(|p| p.0.get_key().is_some() && p.0.get_key().unwrap().get_value() == id)
    }

    pub fn contains(&self, key: &RegistryKey<T>) -> bool {
        self.entries
            .iter()
            .any(|p| p.0.get_key().is_some() && p.0.get_key().unwrap() == key)
    }

    pub fn freeze(&mut self) {
        self.frozen = true;
    }

    pub fn set(
        &mut self,
        id: usize,
        key: RegistryKey<T>,
        object: T,
        lifecycle: Lifecycle,
    ) -> Option<usize> {
        if self.frozen || self.entries.len() < id {
            error!("Registry is already frozen (trying to add key {})", key);
            return None;
        }
        if self.entries.len() != id {
            self.entries.remove(id);
        }
        self.entries.insert(
            id,
            (
                {
                    let mut reference = ReferenceEntry::<T>::stand_alone(Some(object));
                    reference.set_registry_key(key);
                    RegistryEntry::Reference(reference)
                },
                lifecycle,
            ),
        );
        self.lifecycle = self.lifecycle + lifecycle;
        Some(id)
    }

    pub fn add(&mut self, key: RegistryKey<T>, object: T, lifecycle: Lifecycle) -> Option<usize> {
        self.set(self.entries.len(), key, object, lifecycle)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get_default_id(&self) -> &Identifier {
        self.default_id.as_ref().unwrap()
    }

    pub fn get_from_id_default(&self, id: &Identifier) -> &T {
        self.get_from_id(id)
            .unwrap_or(self.get_from_id(self.get_default_id()).unwrap())
    }

    pub fn get_from_raw_id_default(&self, id: usize) -> &T {
        self.get_from_raw_id(id)
            .unwrap_or(self.get_from_id(self.get_default_id()).unwrap())
    }

    pub fn get_default_raw_id(&self) -> usize {
        self.get_raw_id_from_id(self.get_default_id()).unwrap()
    }
}

impl<T> IndexedIterable<T> for Registry<T> {
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
        self.entries.iter().filter_map(|e| e.0.value()).collect()
    }

    fn vec_mut(&mut self) -> Vec<&mut T> {
        self.entries
            .iter_mut()
            .filter_map(|e| e.0.value_mut())
            .collect()
    }
}
